mod store;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
};
use rustrict::CensorStr;
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::info;

struct AppState {
    store: store::ScoreStore,
    secret: String,
}

#[derive(Deserialize)]
struct SubmitScore {
    name: String,
    score: u32,
}

async fn get_scores(State(state): State<Arc<AppState>>) -> Json<store::ScoreBuckets> {
    Json(state.store.get_buckets().await)
}

async fn post_score(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SubmitScore>,
) -> StatusCode {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let expected = format!("Bearer {}", state.secret);
    if auth != expected {
        return StatusCode::UNAUTHORIZED;
    }

    let name = payload.name.censor();
    state.store.add(name, payload.score).await;
    StatusCode::OK
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive("scores_server=debug".parse().unwrap())
                .from_env_lossy(),
        )
        .init();

    let secret = std::env::var("SCORES_SECRET").unwrap_or_else(|_| "dev-secret".to_string());
    let scores_file = std::env::var("SCORES_FILE").unwrap_or_else(|_| "scores.json".to_string());

    let state = Arc::new(AppState {
        store: store::ScoreStore::new(PathBuf::from(scores_file)),
        secret,
    });

    let app = Router::new()
        .route("/api/scores", get(get_scores).post(post_score))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 9003));
    info!("Scores server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
