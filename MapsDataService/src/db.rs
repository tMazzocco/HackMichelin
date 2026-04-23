use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{Restaurant, RestaurantNearby},
};

/// Bounding-box pre-filter then exact Haversine check.
/// LEFT JOIN restaurant_stats inlines good_pct and total_posts so the map
/// overlay never needs a second round-trip.
/// $1 = lat, $2 = lng, $3 = radius_meters, $4 = limit
const NEARBY_SQL: &str = r#"
    SELECT
        r.id, r.name, r.slug, r.chef, r.lat, r.lng,
        r.city, r.country_code, r.country_name, r.region_name, r.area_name,
        r.street, r.postcode, r.phone, r.website, r.short_link,
        r.michelin_award, r.distinction_score, r.guide_year, r.green_star,
        r.price_category_label, r.main_image_url,
        r.online_booking, r.take_away, r.delivery,
        (
            6371000.0 * acos(
                LEAST(1.0,
                    cos(radians($1)) * cos(radians(r.lat)) * cos(radians(r.lng) - radians($2))
                    + sin(radians($1)) * sin(radians(r.lat))
                )
            )
        ) AS distance_meters,
        COALESCE(s.total_posts, 0)                                              AS total_posts,
        COALESCE(s.good_posts::float / NULLIF(s.total_posts, 0), 0.0)          AS good_pct
    FROM restaurants r
    LEFT JOIN restaurant_stats s ON s.restaurant_id = r.id
    WHERE
        r.lat IS NOT NULL
        AND r.lng IS NOT NULL
        -- fast bounding-box cull (1° lat ≈ 111 320 m)
        AND r.lat BETWEEN $1 - ($3 / 111320.0)
                      AND $1 + ($3 / 111320.0)
        AND r.lng BETWEEN $2 - ($3 / (111320.0 * GREATEST(cos(radians($1)), 0.0001)))
                      AND $2 + ($3 / (111320.0 * GREATEST(cos(radians($1)), 0.0001)))
        -- exact Haversine check
        AND (
            6371000.0 * acos(
                LEAST(1.0,
                    cos(radians($1)) * cos(radians(r.lat)) * cos(radians(r.lng) - radians($2))
                    + sin(radians($1)) * sin(radians(r.lat))
                )
            )
        ) <= $3
    ORDER BY distance_meters
    LIMIT $4
"#;

const BY_ID_SQL: &str = r#"
    SELECT
        id, name, slug, chef, lat, lng,
        city, country_code, country_name, region_name, area_name,
        street, postcode, phone, website, short_link,
        michelin_award, distinction_score, guide_year, green_star,
        price_category_label, main_image_url, main_desc,
        online_booking, take_away, delivery
    FROM restaurants
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
