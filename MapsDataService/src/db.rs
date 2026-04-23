use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{CreateRestaurantPayload, Restaurant, RestaurantNearby, UpdateRestaurantPayload},
};

/// Bounding-box pre-filter then exact Haversine check.
/// JOINs cities, countries and michelin_awards so callers get flat string
/// fields without extra round-trips.
/// LEFT JOIN restaurant_stats inlines good_pct and total_posts so the map
/// overlay never needs a second round-trip.
/// $1 = lat, $2 = lng, $3 = radius_meters, $4 = limit
const NEARBY_SQL: &str = r#"
    SELECT
        r.id, r.name, r.slug, r.chef,
        r.latitude, r.longitude,
        ci.name                                                       AS city,
        co.code                                                       AS country_code,
        co.name                                                       AS country_name,
        ci.region_name,
        ci.area_name,
        r.street, r.postcode, r.phone, r.website, r.short_link,
        ma.michelin_award,
        r.distinction_score, r.guide_year, r.green_star,
        (
            SELECT pc.label
            FROM costing c
            JOIN price_categories pc ON pc.code = c.price_categories_code
            WHERE c.restaurant_id = r.id
            LIMIT 1
        )                                                             AS price_category_label,
        r.main_image_url,
        r.online_booking, r.take_away, r.delivery,
        (
            6371000.0 * acos(
                LEAST(1.0,
                    cos(radians($1)) * cos(radians(r.latitude)) * cos(radians(r.longitude) - radians($2))
                    + sin(radians($1)) * sin(radians(r.latitude))
                )
            )
        )                                                             AS distance_meters,
        COALESCE(s.total_posts, 0)                                    AS total_posts,
        COALESCE(s.good_posts::float / NULLIF(s.total_posts, 0), 0.0) AS good_pct
    FROM restaurants r
    JOIN  cities          ci ON ci.id = r.city_id
    JOIN  countries       co ON co.id = ci.country_id
    LEFT JOIN michelin_awards ma ON ma.id = r.michelin_award_id
    LEFT JOIN restaurant_stats s  ON s.restaurant_id = r.id
    WHERE
        r.latitude  IS NOT NULL
        AND r.longitude IS NOT NULL
        -- fast bounding-box cull (1° lat ≈ 111 320 m)
        AND r.latitude  BETWEEN $1 - ($3 / 111320.0)
                            AND $1 + ($3 / 111320.0)
        AND r.longitude BETWEEN $2 - ($3 / (111320.0 * GREATEST(cos(radians($1)), 0.0001)))
                            AND $2 + ($3 / (111320.0 * GREATEST(cos(radians($1)), 0.0001)))
        -- exact Haversine check
        AND (
            6371000.0 * acos(
                LEAST(1.0,
                    cos(radians($1)) * cos(radians(r.latitude)) * cos(radians(r.longitude) - radians($2))
                    + sin(radians($1)) * sin(radians(r.latitude))
                )
            )
        ) <= $3
    ORDER BY distance_meters
    LIMIT $4
"#;

const BY_ID_SQL: &str = r#"
    SELECT
        r.id, r.name, r.slug, r.chef,
        r.latitude, r.longitude,
        ci.name   AS city,
        co.code   AS country_code,
        co.name   AS country_name,
        ci.region_name,
        ci.area_name,
        r.street, r.postcode, r.phone, r.website, r.short_link,
        ma.michelin_award,
        r.distinction_score, r.guide_year, r.green_star,
        (
            SELECT pc.label
            FROM costing c
            JOIN price_categories pc ON pc.code = c.price_categories_code
            WHERE c.restaurant_id = r.id
            LIMIT 1
        )         AS price_category_label,
        r.main_image_url, r.main_desc,
        r.online_booking, r.take_away, r.delivery
    FROM restaurants r
    JOIN  cities          ci ON ci.id = r.city_id
    JOIN  countries       co ON co.id = ci.country_id
    LEFT JOIN michelin_awards ma ON ma.id = r.michelin_award_id
    WHERE r.id = $1
"#;

const LIST_SQL: &str = r#"
    SELECT
        id, name, slug, chef, lat, lng,
        city, country_code, country_name, region_name, area_name,
        street, postcode, phone, website, short_link,
        michelin_award, distinction_score, guide_year, green_star,
        price_category_label, main_image_url, main_desc,
        online_booking, take_away, delivery
    FROM restaurants
    ORDER BY name ASC
    LIMIT $1
"#;

const CREATE_SQL: &str = r#"
    INSERT INTO restaurants (
        id, name, slug, chef, lat, lng,
        city, country_code, country_name, region_name, area_name,
        street, postcode, phone, website, short_link,
        michelin_award, distinction_score, guide_year, green_star,
        price_category_label, main_image_url, main_desc,
        online_booking, take_away, delivery
    )
    VALUES (
        $1, $2, $3, $4, $5, $6,
        $7, $8, $9, $10, $11,
        $12, $13, $14, $15, $16,
        $17, $18, $19, $20,
        $21, $22, $23,
        $24, $25, $26
    )
    RETURNING
        id, name, slug, chef, lat, lng,
        city, country_code, country_name, region_name, area_name,
        street, postcode, phone, website, short_link,
        michelin_award, distinction_score, guide_year, green_star,
        price_category_label, main_image_url, main_desc,
        online_booking, take_away, delivery
"#;

const UPDATE_SQL: &str = r#"
    UPDATE restaurants
    SET
        name = $2,
        slug = $3,
        chef = $4,
        lat = $5,
        lng = $6,
        city = $7,
        country_code = $8,
        country_name = $9,
        region_name = $10,
        area_name = $11,
        street = $12,
        postcode = $13,
        phone = $14,
        website = $15,
        short_link = $16,
        michelin_award = $17,
        distinction_score = $18,
        guide_year = $19,
        green_star = $20,
        price_category_label = $21,
        main_image_url = $22,
        main_desc = $23,
        online_booking = $24,
        take_away = $25,
        delivery = $26,
        last_updated = NOW()
    WHERE id = $1
    RETURNING
        id, name, slug, chef, lat, lng,
        city, country_code, country_name, region_name, area_name,
        street, postcode, phone, website, short_link,
        michelin_award, distinction_score, guide_year, green_star,
        price_category_label, main_image_url, main_desc,
        online_booking, take_away, delivery
"#;

const DELETE_SQL: &str = r#"
    DELETE FROM restaurants
    WHERE id = $1
"#;

pub async fn get_nearby(
    pool:          &PgPool,
    lat:           f64,
    lng:           f64,
    radius_meters: f64,
    limit:         i64,
) -> Result<Vec<RestaurantNearby>, AppError> {
    let rows = sqlx::query_as::<_, RestaurantNearby>(NEARBY_SQL)
        .bind(lat)
        .bind(lng)
        .bind(radius_meters)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_by_id(pool: &PgPool, id: &str) -> Result<Option<Restaurant>, AppError> {
    let row = sqlx::query_as::<_, Restaurant>(BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_restaurants(pool: &PgPool, limit: i64) -> Result<Vec<Restaurant>, AppError> {
    let rows = sqlx::query_as::<_, Restaurant>(LIST_SQL)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn create_restaurant(
    pool: &PgPool,
    payload: CreateRestaurantPayload,
) -> Result<Restaurant, AppError> {
    let row = sqlx::query_as::<_, Restaurant>(CREATE_SQL)
        .bind(payload.id)
        .bind(payload.name)
        .bind(payload.slug)
        .bind(payload.chef)
        .bind(payload.lat)
        .bind(payload.lng)
        .bind(payload.city)
        .bind(payload.country_code)
        .bind(payload.country_name)
        .bind(payload.region_name)
        .bind(payload.area_name)
        .bind(payload.street)
        .bind(payload.postcode)
        .bind(payload.phone)
        .bind(payload.website)
        .bind(payload.short_link)
        .bind(payload.michelin_award)
        .bind(payload.distinction_score)
        .bind(payload.guide_year)
        .bind(payload.green_star)
        .bind(payload.price_category_label)
        .bind(payload.main_image_url)
        .bind(payload.main_desc)
        .bind(payload.online_booking)
        .bind(payload.take_away)
        .bind(payload.delivery)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn update_restaurant(
    pool: &PgPool,
    id: &str,
    payload: UpdateRestaurantPayload,
) -> Result<Option<Restaurant>, AppError> {
    let row = sqlx::query_as::<_, Restaurant>(UPDATE_SQL)
        .bind(id)
        .bind(payload.name)
        .bind(payload.slug)
        .bind(payload.chef)
        .bind(payload.lat)
        .bind(payload.lng)
        .bind(payload.city)
        .bind(payload.country_code)
        .bind(payload.country_name)
        .bind(payload.region_name)
        .bind(payload.area_name)
        .bind(payload.street)
        .bind(payload.postcode)
        .bind(payload.phone)
        .bind(payload.website)
        .bind(payload.short_link)
        .bind(payload.michelin_award)
        .bind(payload.distinction_score)
        .bind(payload.guide_year)
        .bind(payload.green_star)
        .bind(payload.price_category_label)
        .bind(payload.main_image_url)
        .bind(payload.main_desc)
        .bind(payload.online_booking)
        .bind(payload.take_away)
        .bind(payload.delivery)
        .fetch_optional(pool)
        .await?;

    Ok(row)
}

pub async fn delete_restaurant(pool: &PgPool, id: &str) -> Result<u64, AppError> {
    let result = sqlx::query(DELETE_SQL)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}