use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct LlmClient {
    http: Client,
    pub provider: LlmProvider,
}

#[derive(Clone, Debug)]
pub enum LlmProvider {
    Ollama { base_url: String, model: String },
    OpenAICompatible { base_url: String, api_key: String, model: String, name: String },
}

// ── Ollama types ──────────────────────────────────────────────

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

// ── OpenAI-compatible types (works with OpenAI, Anthropic, Groq, Azure, Together, vLLM, LM Studio) ──

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageOut,
}

#[derive(Deserialize)]
struct ChatMessageOut {
    content: String,
}

impl LlmClient {
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            http: Client::new(),
            provider,
        }
    }

    /// Build from environment variables.
    ///
    /// Supported providers (set via `LLM_PROVIDER`):
    ///
    /// | Provider   | LLM_PROVIDER value | Required env vars                     |
    /// |------------|-------------------|---------------------------------------|
    /// | Ollama     | `ollama`          | `OLLAMA_URL`, `OLLAMA_MODEL`          |
    /// | OpenAI     | `openai`          | `OPENAI_API_KEY`, `OPENAI_MODEL`      |
    /// | Anthropic  | `anthropic`       | `ANTHROPIC_API_KEY`, `ANTHROPIC_MODEL`|
    /// | Groq       | `groq`            | `GROQ_API_KEY`, `GROQ_MODEL`          |
    /// | Together   | `together`        | `TOGETHER_API_KEY`, `TOGETHER_MODEL`  |
    /// | Custom     | `custom`          | `CUSTOM_LLM_URL`, `CUSTOM_LLM_KEY`, `CUSTOM_LLM_MODEL` |
    ///
    /// The "custom" provider works with any OpenAI-compatible API (vLLM, LM Studio, Azure, etc.)
    pub fn from_env() -> Self {
        let provider_str = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "ollama".into());

        let provider = match provider_str.to_lowercase().as_str() {
            "openai" => LlmProvider::OpenAICompatible {
                base_url: "https://api.openai.com/v1".into(),
                api_key: std::env::var("OPENAI_API_KEY")
                    .expect("OPENAI_API_KEY required when LLM_PROVIDER=openai"),
                model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into()),
                name: "OpenAI".into(),
            },
            "anthropic" => LlmProvider::OpenAICompatible {
                base_url: std::env::var("ANTHROPIC_BASE_URL")
                    .unwrap_or_else(|_| "https://api.anthropic.com/v1".into()),
                api_key: std::env::var("ANTHROPIC_API_KEY")
                    .expect("ANTHROPIC_API_KEY required when LLM_PROVIDER=anthropic"),
                model: std::env::var("ANTHROPIC_MODEL")
                    .unwrap_or_else(|_| "claude-sonnet-4-20250514".into()),
                name: "Anthropic".into(),
            },
            "groq" => LlmProvider::OpenAICompatible {
                base_url: "https://api.groq.com/openai/v1".into(),
                api_key: std::env::var("GROQ_API_KEY")
                    .expect("GROQ_API_KEY required when LLM_PROVIDER=groq"),
                model: std::env::var("GROQ_MODEL")
                    .unwrap_or_else(|_| "llama-3.1-70b-versatile".into()),
                name: "Groq".into(),
            },
            "together" => LlmProvider::OpenAICompatible {
                base_url: "https://api.together.xyz/v1".into(),
                api_key: std::env::var("TOGETHER_API_KEY")
                    .expect("TOGETHER_API_KEY required when LLM_PROVIDER=together"),
                model: std::env::var("TOGETHER_MODEL")
                    .unwrap_or_else(|_| "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo".into()),
                name: "Together".into(),
            },
            "custom" => LlmProvider::OpenAICompatible {
                base_url: std::env::var("CUSTOM_LLM_URL")
                    .expect("CUSTOM_LLM_URL required when LLM_PROVIDER=custom"),
                api_key: std::env::var("CUSTOM_LLM_KEY").unwrap_or_default(),
                model: std::env::var("CUSTOM_LLM_MODEL")
                    .expect("CUSTOM_LLM_MODEL required when LLM_PROVIDER=custom"),
                name: std::env::var("CUSTOM_LLM_NAME").unwrap_or_else(|_| "Custom".into()),
            },
            _ => LlmProvider::Ollama {
                base_url: std::env::var("OLLAMA_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".into()),
                model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.1:8b".into()),
            },
        };

        Self::new(provider)
    }

    pub fn provider_name(&self) -> String {
        match &self.provider {
            LlmProvider::Ollama { model, .. } => format!("Ollama ({})", model),
            LlmProvider::OpenAICompatible { name, model, .. } => format!("{} ({})", name, model),
        }
    }

    /// Send a prompt to the configured LLM and return the raw text response.
    pub async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        match &self.provider {
            LlmProvider::Ollama { base_url, model } => {
                let url = format!("{}/api/generate", base_url);
                let full_prompt = format!("{}\n\n{}", system_prompt, user_prompt);
                let body = OllamaRequest {
                    model: model.clone(),
                    prompt: full_prompt,
                    stream: false,
                };

                let resp = self
                    .http
                    .post(&url)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Ollama request failed: {e}"))?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("Ollama returned {status}: {text}"));
                }

                let data: OllamaResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse Ollama response: {e}"))?;

                Ok(data.response)
            }
            LlmProvider::OpenAICompatible { base_url, api_key, model, .. } => {
                let url = format!("{}/chat/completions", base_url);
                let body = ChatRequest {
                    model: model.clone(),
                    messages: vec![
                        ChatMessage {
                            role: "system".into(),
                            content: system_prompt.to_string(),
                        },
                        ChatMessage {
                            role: "user".into(),
                            content: user_prompt.to_string(),
                        },
                    ],
                    temperature: 0.2,
                };

                let mut req = self.http.post(&url).json(&body);
                if !api_key.is_empty() {
                    req = req.bearer_auth(api_key);
                }

                let resp = req
                    .send()
                    .await
                    .map_err(|e| format!("LLM request failed: {e}"))?;

                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("LLM returned {status}: {text}"));
                }

                let data: ChatResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse LLM response: {e}"))?;

                data.choices
                    .into_iter()
                    .next()
                    .map(|c| c.message.content)
                    .ok_or_else(|| "No choices in LLM response".into())
            }
        }
    }
}
