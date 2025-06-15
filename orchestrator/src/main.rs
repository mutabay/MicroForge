use axum::{routing::get, Router, serve};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::net::TcpListener;

use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ServiceInfo {
    name: &'static str,
    url:  &'static str,
}

async fn list_services() -> axum::Json<Vec<ServiceInfo>> {
    axum::Json(vec![
        ServiceInfo { name: "user_service", url: "http://localhost:8001" },
        // later: more services get registered here
    ])
}

#[tokio::main]
async fn main() {
    // structured logs
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // define routes
    let app = Router::new().route("/", get(root));

    // bind & serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("🚀 Orchestrator listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app)
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Welcome to MicroForge Orchestrator 👋"
}
