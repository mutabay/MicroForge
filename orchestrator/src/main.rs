mod forge;
mod llm;
mod pipeline;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use llm::LlmClient;
use pipeline::{PipelineRequest, PipelineStore};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ── Shared application state ──────────────────────────────────

#[derive(Clone)]
struct AppState {
    llm: LlmClient,
    pipelines: PipelineStore,
}

// ── Service registry ──────────────────────────────────────────

#[derive(Serialize)]
struct ServiceInfo {
    name: &'static str,
    url: &'static str,
}

async fn list_services() -> Json<Vec<ServiceInfo>> {
    Json(vec![ServiceInfo {
        name: "user_service",
        url: "http://localhost:8001",
    }])
}

async fn health() -> &'static str {
    "OK"
}

async fn root() -> &'static str {
    "Welcome to MicroForge Orchestrator 🔥"
}

// ── Pipeline endpoints ────────────────────────────────────────

async fn run_pipeline_handler(
    State(state): State<AppState>,
    Json(req): Json<PipelineRequest>,
) -> impl IntoResponse {
    match pipeline::run_pipeline(req, state.pipelines).await {
        Ok(result) => (axum::http::StatusCode::OK, Json(serde_json::json!(result))).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

async fn get_pipeline_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.pipelines.get(&id).await {
        Some(result) => Json(serde_json::json!(result)).into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn list_pipelines_handler(State(state): State<AppState>) -> impl IntoResponse {
    let pipelines = state.pipelines.list().await;
    Json(serde_json::json!(pipelines))
}

// ── Forge endpoint (wraps LLM from shared state) ─────────────

async fn forge_handler(
    State(state): State<AppState>,
    Json(payload): Json<forge::ForgeRequest>,
) -> impl IntoResponse {
    forge::forge_service(State(state.llm), Json(payload)).await
}

// ── Main ──────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        llm: LlmClient::from_env(),
        pipelines: PipelineStore::default(),
    };

    tracing::info!("LLM provider: {}", state.llm.provider_name());

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/services", get(list_services))
        .route("/forge", post(forge_handler))
        .route(
            "/pipelines",
            post(run_pipeline_handler).get(list_pipelines_handler),
        )
        .route("/pipelines/{id}", get(get_pipeline_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("🚀 Orchestrator listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
