use serde::Serialize;
use serde_json::{json, Value};

use crate::models::Restaurant;

const ELASTICSEARCH_URL: &str = "http://localhost:9200";

#[derive(Debug, Serialize)]
pub struct SearchRestaurantResult {
    pub id: String,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub michelin_award: Option<String>,
    pub price_category_label: Option<String>,
    pub image: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub distance_meters: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub q: String,
    pub city: Option<String>,
    pub award: Option<String>,
    pub price: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_meters: Option<f64>,
    pub limit: Option<i64>,
}

fn restaurant_to_es_doc(restaurant: &Restaurant) -> Value {
    json!({
        "id": restaurant.id,
        "name": restaurant.name,
        "slug": restaurant.slug,
        "chef": restaurant.chef,
        "city": restaurant.city,
        "country": restaurant.country_name,
        "country_code": restaurant.country_code,
        "region": restaurant.region_name,
        "street": restaurant.street,
        "postcode": restaurant.postcode,
        "phone": restaurant.phone,
        "website": restaurant.website,
        "short_link": restaurant.short_link,
        "michelin_award": restaurant.michelin_award,
        "michelin_star": restaurant.michelin_award,
        "distinction_score": restaurant.distinction_score,
        "guide_year": restaurant.guide_year,
        "green_star": restaurant.green_star,
        "price_category_label": restaurant.price_category_label,
        "image": restaurant.main_image_url,
        "description": restaurant.main_desc,
        "online_booking": restaurant.online_booking,
        "take_away": restaurant.take_away,
        "delivery": restaurant.delivery,
        "location": match (restaurant.lat, restaurant.lng) {
            (Some(lat), Some(lng)) => json!({ "lat": lat, "lon": lng }),
            _ => Value::Null,
        }
    })
}

fn normalized_price_aliases(input: &str) -> Vec<String> {
    match input.trim().to_lowercase().as_str() {
        "budget" | "cheap" | "low" | "petit-budget" | "pas-cher" => vec![
            "budget".to_string(),
            "A small spend".to_string(),
            "small spend".to_string(),
            "affordable".to_string(),
        ],
        "moderate" | "mid" | "mid-range" | "moyen" | "modere" => vec![
            "mid-range".to_string(),
            "A moderate spend".to_string(),
            "moderate".to_string(),
        ],
        "premium" | "special" | "occasion" => vec![
            "premium".to_string(),
            "Special occasion".to_string(),
        ],
        "luxury" | "high" | "luxe" => vec![
            "luxury".to_string(),
            "Spare no expense".to_string(),
        ],
        other => vec![other.to_string()],
    }
}

pub async fn search_with_filters(
    filters: SearchFilters,
) -> Result<Vec<SearchRestaurantResult>, reqwest::Error> {
    let client = reqwest::Client::new();

    let limit = filters.limit.unwrap_or(20).clamp(1, 100);

    let mut should_clauses = Vec::new();
    let mut filter_clauses = Vec::new();

    let q = filters.q.trim();

    if !q.is_empty() {
        should_clauses.push(json!({
            "match_phrase": {
                "name": {
                    "query": q,
                    "boost": 10.0
                }
            }
        }));

        should_clauses.push(json!({
            "match": {
                "name": {
                    "query": q,
                    "fuzziness": "AUTO",
                    "boost": 6.0
                }
            }
        }));

        should_clauses.push(json!({
            "multi_match": {
                "query": q,
                "fields": [
                    "name^4",
                    "description^2",
                    "city^3"
                ],
                "fuzziness": "AUTO",
                "operator": "and"
            }
        }));

        should_clauses.push(json!({
            "match": {
                "cuisines": {
                    "query": q,
                    "fuzziness": "AUTO",
                    "boost": 5.0
                }
            }
        }));

        should_clauses.push(json!({
            "match": {
                "cuisine_slugs": {
                    "query": q,
                    "fuzziness": "AUTO",
                    "boost": 5.0
                }
            }
        }));
    }

    if let Some(city) = filters.city.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        filter_clauses.push(json!({
            "term": {
                "city": city
            }
        }));
    }

    if let Some(award) = filters.award.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        filter_clauses.push(json!({
            "term": {
                "michelin_award": award
            }
        }));
    }

    if let Some(price) = filters.price.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let aliases = normalized_price_aliases(price);

        let mut price_should = Vec::new();

        for alias in aliases {
            price_should.push(json!({
                "term": { "price_category_slug": alias }
            }));
            price_should.push(json!({
                "term": { "price_category_label": alias }
            }));
            price_should.push(json!({
                "term": { "price_category": alias }
            }));
        }

        filter_clauses.push(json!({
            "bool": {
                "should": price_should,
                "minimum_should_match": 1
            }
        }));
    }

    let has_geo = matches!((filters.lat, filters.lng), (Some(_), Some(_)));

    if let (Some(lat), Some(lng)) = (filters.lat, filters.lng) {
        let radius = filters.radius_meters.unwrap_or(1000.0).clamp(1.0, 50_000.0);

        filter_clauses.push(json!({
            "geo_distance": {
                "distance": format!("{}m", radius),
                "location": { "lat": lat, "lon": lng }
            }
        }));
    }

    let query = if should_clauses.is_empty() {
        json!({
            "bool": {
                "filter": filter_clauses
            }
        })
    } else {
        json!({
            "bool": {
                "should": should_clauses,
                "minimum_should_match": 1,
                "filter": filter_clauses
            }
        })
    };

    let mut body = json!({
        "size": limit,
        "query": query
    });

    if let (Some(lat), Some(lng)) = (filters.lat, filters.lng) {
        body["sort"] = json!([
            {
                "_geo_distance": {
                    "location": { "lat": lat, "lon": lng },
                    "order": "asc",
                    "unit": "m"
                }
            }
        ]);
    }

    let response = client
        .post(format!("{}/restaurants/_search", ELASTICSEARCH_URL))
        .json(&body)
        .send()
        .await?;

    let json = response.json::<Value>().await?;

    let results = json["hits"]["hits"]
        .as_array()
        .map(|hits| {
            hits.iter()
                .filter_map(|hit| {
                    let source = &hit["_source"];

                    let id = source["id"]
                        .as_str()
                        .or_else(|| source["objectID"].as_str())?
                        .to_string();

                    let name = source["name"].as_str()?.to_string();
                    let city = source["city"].as_str().map(|s| s.to_string());

                    let country = source["country"]
                        .as_str()
                        .or_else(|| source["country_name"].as_str())
                        .map(|s| s.to_string());

                    let michelin_award =
                        source["michelin_award"].as_str().map(|s| s.to_string());

                    let price_category_label = source["price_category_label"]
                        .as_str()
                        .or_else(|| source["price_category"].as_str())
                        .map(|s| s.to_string());

                    let image = source["image"]
                        .as_str()
                        .or_else(|| source["main_image_url"].as_str())
                        .map(|s| s.to_string());

                    let lat = source["location"]["lat"].as_f64();
                    let lng = source["location"]["lon"].as_f64();

                    let distance_meters = if has_geo {
                        hit["sort"]
                            .as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_f64())
                    } else {
                        None
                    };

                    Some(SearchRestaurantResult {
                        id,
                        name,
                        city,
                        country,
                        michelin_award,
                        price_category_label,
                        image,
                        lat,
                        lng,
                        distance_meters,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(results)
}

pub async fn search_free_text(query: &str) -> Result<Vec<SearchRestaurantResult>, reqwest::Error> {
    search_with_filters(SearchFilters {
        q: query.to_string(),
        city: None,
        award: None,
        price: None,
        lat: None,
        lng: None,
        radius_meters: None,
        limit: Some(20),
    })
    .await
}

pub async fn index_restaurant(restaurant: &Restaurant) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let doc = restaurant_to_es_doc(restaurant);

    client
        .put(format!("{}/restaurants/_doc/{}", ELASTICSEARCH_URL, restaurant.id))
        .json(&doc)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn delete_restaurant(id: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let response = client
        .delete(format!("{}/restaurants/_doc/{}", ELASTICSEARCH_URL, id))
        .send()
        .await?;

    let status = response.status();

    if status.is_success() || status.as_u16() == 404 {
        Ok(())
    } else {
        response.error_for_status()?;
        Ok(())
    }
}