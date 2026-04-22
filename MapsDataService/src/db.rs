use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{Restaurant, RestaurantNearby},
};

/// Bounding-box pre-filter then exact Haversine check.
/// $1 = lat, $2 = lng, $3 = radius_meters, $4 = limit
const NEARBY_SQL: &str = r#"
    SELECT
        id, name, slug, chef, lat, lng,
        city, country_code, country_name, region_name, area_name,
        street, postcode, phone, website, short_link,
        michelin_award, distinction_score, guide_year, green_star,
        price_category_label, main_image_url,
        online_booking, take_away, delivery,
        (
            6371000.0 * acos(
                LEAST(1.0,
                    cos(radians($1)) * cos(radians(lat)) * cos(radians(lng) - radians($2))
                    + sin(radians($1)) * sin(radians(lat))
                )
            )
        ) AS distance_meters
    FROM restaurants
    WHERE
        lat IS NOT NULL
        AND lng IS NOT NULL
        -- fast bounding-box cull (1° lat ≈ 111 320 m)
        AND lat BETWEEN $1 - ($3 / 111320.0)
                    AND $1 + ($3 / 111320.0)
        AND lng BETWEEN $2 - ($3 / (111320.0 * GREATEST(cos(radians($1)), 0.0001)))
                    AND $2 + ($3 / (111320.0 * GREATEST(cos(radians($1)), 0.0001)))
        -- exact Haversine check
        AND (
            6371000.0 * acos(
                LEAST(1.0,
                    cos(radians($1)) * cos(radians(lat)) * cos(radians(lng) - radians($2))
                    + sin(radians($1)) * sin(radians(lat))
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
