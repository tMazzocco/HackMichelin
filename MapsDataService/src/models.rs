use serde::Serialize;

/// Full restaurant row returned from a nearby search (includes computed distance).
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RestaurantNearby {
    pub id:                   String,
    pub name:                 String,
    pub slug:                 Option<String>,
    pub chef:                 Option<String>,
    pub lat:                  Option<f64>,
    pub lng:                  Option<f64>,
    pub city:                 Option<String>,
    pub country_code:         Option<String>,
    pub country_name:         Option<String>,
    pub region_name:          Option<String>,
    pub area_name:            Option<String>,
    pub street:               Option<String>,
    pub postcode:             Option<String>,
    pub phone:                Option<String>,
    pub website:              Option<String>,
    pub short_link:           Option<String>,
    pub michelin_award:       Option<String>,
    pub distinction_score:    Option<i32>,
    pub guide_year:           Option<i32>,
    pub green_star:           Option<bool>,
    pub price_category_label: Option<String>,
    pub main_image_url:       Option<String>,
    pub online_booking:       Option<bool>,
    pub take_away:            Option<bool>,
    pub delivery:             Option<bool>,
    /// Haversine distance in metres from the query point.
    pub distance_meters:      f64,
}

/// Restaurant row returned for a single get-by-id lookup.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Restaurant {
    pub id:                   String,
    pub name:                 String,
    pub slug:                 Option<String>,
    pub chef:                 Option<String>,
    pub lat:                  Option<f64>,
    pub lng:                  Option<f64>,
    pub city:                 Option<String>,
    pub country_code:         Option<String>,
    pub country_name:         Option<String>,
    pub region_name:          Option<String>,
    pub area_name:            Option<String>,
    pub street:               Option<String>,
    pub postcode:             Option<String>,
    pub phone:                Option<String>,
    pub website:              Option<String>,
    pub short_link:           Option<String>,
    pub michelin_award:       Option<String>,
    pub distinction_score:    Option<i32>,
    pub guide_year:           Option<i32>,
    pub green_star:           Option<bool>,
    pub price_category_label: Option<String>,
    pub main_image_url:       Option<String>,
    pub main_desc:            Option<String>,
    pub online_booking:       Option<bool>,
    pub take_away:            Option<bool>,
    pub delivery:             Option<bool>,
}
