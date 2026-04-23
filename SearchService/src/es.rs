use elasticsearch::{
    http::transport::Transport, Elasticsearch, IndexParts, SearchParts, UpdateParts,
};
use serde_json::{json, Value};

use crate::error::AppError;

/// Build an Elasticsearch client from a URL string.
pub fn build_client(es_host: &str) -> Result<Elasticsearch, AppError> {
    let transport = Transport::single_node(es_host)
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;
    Ok(Elasticsearch::new(transport))
}

/// Search the `restaurants` index with optional full-text and facet filters.
pub async fn search_restaurants(
    es: &Elasticsearch,
    q: Option<String>,
    city: Option<String>,
    country: Option<String>,
    award: Option<String>,
    _lat: Option<f64>,
    _lng: Option<f64>,
    _radius: Option<String>,
) -> Result<Vec<Value>, AppError> {
    // Build must clause (full-text)
    let must_clause: Value = match &q {
        Some(text) if !text.trim().is_empty() => json!({
            "multi_match": {
                "query": text,
                "fields": ["name^3", "chef^2", "city", "country_name", "region_name"]
            }
        }),
        _ => json!({ "match_all": {} }),
    };

    // Build filter clauses (facets)
    let mut filters: Vec<Value> = Vec::new();

    if let Some(c) = city {
        if !c.is_empty() {
            filters.push(json!({ "term": { "city": c } }));
        }
    }
    if let Some(c) = country {
        if !c.is_empty() {
            filters.push(json!({ "term": { "country_code": c } }));
        }
    }
    if let Some(a) = award {
        if !a.is_empty() {
            filters.push(json!({ "term": { "michelin_award": a } }));
        }
    }

    let query = if filters.is_empty() {
        json!({ "query": must_clause })
    } else {
        json!({
            "query": {
                "bool": {
                    "must": must_clause,
                    "filter": filters
                }
            }
        })
    };

    let body = json!({
        "size": 20,
        "query": query["query"]
    });

    let response = es
        .search(SearchParts::Index(&["restaurants"]))
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;

    let response_body: Value = response
        .json()
        .await
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;

    extract_sources(&response_body)
}

/// Search the `users` index with a full-text query.
pub async fn search_users(es: &Elasticsearch, q: String) -> Result<Vec<Value>, AppError> {
    let body = json!({
        "size": 20,
        "query": {
            "multi_match": {
                "query": q,
                "fields": ["username^2", "bio"]
            }
        }
    });

    let response = es
        .search(SearchParts::Index(&["users"]))
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;

    let response_body: Value = response
        .json()
        .await
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;

    extract_sources(&response_body)
}

/// Index (upsert) a user document. Uses PUT with document ID.
pub async fn index_user(es: &Elasticsearch, user_id: &str, doc: Value) -> Result<(), AppError> {
    let response = es
        .index(IndexParts::IndexId("users", user_id))
        .body(doc)
        .send()
        .await
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;

    let status = response.status_code();
    if !status.is_success() {
        let body: Value = response
            .json()
            .await
            .unwrap_or_else(|_| json!({ "error": "unknown" }));
        return Err(AppError::ElasticsearchError(format!(
            "index_user failed ({status}): {body}"
        )));
    }

    Ok(())
}

/// Update an existing user document (partial update via `doc`).
pub async fn update_user(es: &Elasticsearch, user_id: &str, doc: Value) -> Result<(), AppError> {
    let body = json!({ "doc": doc });

    let response = es
        .update(UpdateParts::IndexId("users", user_id))
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::ElasticsearchError(e.to_string()))?;

    let status = response.status_code();
    if !status.is_success() {
        let body: Value = response
            .json()
            .await
            .unwrap_or_else(|_| json!({ "error": "unknown" }));
        return Err(AppError::ElasticsearchError(format!(
            "update_user failed ({status}): {body}"
        )));
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn extract_sources(response_body: &Value) -> Result<Vec<Value>, AppError> {
    let hits = response_body["hits"]["hits"]
        .as_array()
        .ok_or_else(|| AppError::ElasticsearchError("unexpected ES response shape".into()))?;

    let sources: Vec<Value> = hits
        .iter()
        .filter_map(|hit| hit["_source"].as_object().map(|o| Value::Object(o.clone())))
        .collect();

    Ok(sources)
}
