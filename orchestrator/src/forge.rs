use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::llm::LlmClient;

#[derive(Deserialize)]
pub struct ForgeRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct ForgeResponse {
    pub openapi_spec: String,
    pub provider: String,
}

const SYSTEM_PROMPT: &str = r#"You are MicroForge, an expert API architect specializing in microservice design.

Given a high-level description of a microservice, generate a complete OpenAPI 3.0 specification in YAML format.

Rules:
- Include all CRUD endpoints (GET list, GET by id, POST create, PUT update, DELETE)
- Define request/response schemas in components/schemas
- Use proper HTTP status codes
- Add meaningful descriptions
- Return ONLY the YAML content, no markdown fences or explanation
"#;

pub async fn forge_service(
    State(llm): State<LlmClient>,
    Json(payload): Json<ForgeRequest>,
) -> impl IntoResponse {
    match llm.generate(SYSTEM_PROMPT, &payload.prompt).await {
        Ok(spec) => {
            let provider = llm.provider_name();
            Json(ForgeResponse {
                openapi_spec: clean_openapi_spec(&spec),
                provider,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("LLM generation failed: {e}");
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    }
}

fn clean_openapi_spec(spec: &str) -> String {
    let spec = spec.trim();

    if !spec.starts_with("```") {
        return spec.to_string();
    }

    let mut lines = spec.lines();
    let first = lines.next().unwrap_or_default().trim();
    if !matches!(first, "```" | "```yaml" | "```yml") {
        return spec.to_string();
    }

    let mut body: Vec<&str> = lines.collect();
    if body.last().map(|line| line.trim()) == Some("```") {
        body.pop();
    }

    body.join("\n").trim().to_string()
}
