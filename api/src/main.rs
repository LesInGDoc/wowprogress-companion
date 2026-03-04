use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, put},
    Router,
};
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    auth_token: String,
}

#[derive(Debug, Deserialize)]
struct PullsQuery {
    realm_slug: String,
    guild_slug: String,
    difficulty: String,
    #[serde(default)]
    bosses: Option<String>,
    raid_slug: String,
    #[serde(default)]
    hide_rejected: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Pull {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<mongodb::bson::oid::ObjectId>,
    pull_id: i64,
    #[serde(flatten)]
    data: Value,
}

#[derive(Debug, Deserialize)]
struct UpdatePullRequest {
    action: PullAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PullAction {
    Accept,
    Reject,
}

// GET /pulls/filters - Get available filter values
async fn get_filters(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    let collection: Collection<Pull> = state.db.collection("pulls");

    // Get distinct values for each field
    let realm_slugs = collection
        .distinct("realm.slug", doc! {})
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let guild_slugs = collection
        .distinct("guild.slug", doc! {})
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let raid_slugs = collection
        .distinct("encounter.raid.slug", doc! {})
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let boss_slugs = collection
        .distinct("encounter.boss.slug", doc! {})
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let difficulty_ids = collection
        .distinct("encounter.difficulty", doc! {})
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let filters = serde_json::json!({
        "realms": realm_slugs,
        "guilds": guild_slugs,
        "raids": raid_slugs,
        "bosses": boss_slugs,
        "difficulties": difficulty_ids
    });

    Ok(Json(filters))
}

// GET /pulls?realm_slug=X&guild_slug=Y&difficulty=Z&bosses=A,B&raid_slug=R
async fn get_pulls(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PullsQuery>,
) -> Result<Json<Vec<Pull>>, StatusCode> {
    let collection: Collection<Pull> = state.db.collection("pulls");

    // Build the filter query
    let mut filter = doc! {
        "realm.slug": params.realm_slug,
        "guild.slug": params.guild_slug,
        "encounter.difficulty": params.difficulty,
        "encounter.raid.slug": params.raid_slug,
    };

    // Add bosses filter if provided
    if let Some(bosses) = params.bosses {
        let boss_list: Vec<String> = bosses.split(',').map(|s| s.trim().to_string()).collect();
        if !boss_list.is_empty() {
            filter.insert("encounter.boss.slug", doc! { "$in": boss_list });
        }
    }

    // Filter out rejected pulls if hide_rejected is true
    if params.hide_rejected.unwrap_or(false) {
        filter.insert("status", doc! { "$ne": "rejected" });
    }

    // Create find options with sorting by pull_count descending
    let find_options = mongodb::options::FindOptions::builder()
        .sort(doc! { "pull_count": -1 })
        .build();

    let mut cursor = collection.find(filter).with_options(find_options).await.map_err(|e| {
        eprintln!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut pulls = Vec::new();
    use futures::StreamExt;
    while let Some(result) = cursor.next().await {
        match result {
            Ok(pull) => pulls.push(pull),
            Err(e) => {
                eprintln!("Error reading pull: {}", e);
                continue;
            }
        }
    }

    Ok(Json(pulls))
}

// PUT /pulls/:id
async fn update_pull(
    State(state): State<Arc<AppState>>,
    Path(pull_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpdatePullRequest>,
) -> StatusCode {
    // Verify authentication token
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if token != Some(&state.auth_token) {
        return StatusCode::UNAUTHORIZED;
    }

    let collection: Collection<mongodb::bson::Document> = state.db.collection("pulls");

    // Update the pull based on the action
    let update_doc = match payload.action {
        PullAction::Accept => doc! { "$set": { "status": "accepted" } },
        PullAction::Reject => doc! { "$set": { "status": "rejected" } },
    };

    let filter = doc! { "pull_id": pull_id };

    match collection.update_one(filter, update_doc).await {
        Ok(result) => {
            if result.matched_count == 0 {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::OK
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    let mongodb_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = std::env::var("MONGODB_DB_NAME").expect("MONGODB_DB_NAME must be set");
    let auth_token = std::env::var("AUTH_TOKEN").expect("AUTH_TOKEN must be set");
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    // Connect to MongoDB
    let client = Client::with_uri_str(&mongodb_uri)
        .await
        .expect("Failed to connect to MongoDB");

    let db = client.database(&db_name);

    let state = Arc::new(AppState { db, auth_token });

    // Build the router
    let app = Router::new()
        .route("/pulls", get(get_pulls))
        .route("/pulls/filters", get(get_filters))
        .route("/pulls/:id", put(update_pull))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("API server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
