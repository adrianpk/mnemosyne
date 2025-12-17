use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

use super::{LLMResponse, Prompt};
use crate::config::Config;

/// Log LLM responses for debugging parse errors.
fn log_response(content: &str, error: Option<&str>) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/mnemosyne-llm.log")
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "=== {} ===", timestamp);
        let _ = writeln!(file, "{}", content);
        if let Some(e) = error {
            let _ = writeln!(file, "ERROR: {}", e);
        }
        let _ = writeln!(file, "");
    }
}

// Error Types

#[derive(Debug)]
pub enum LLMError {
    MissingApiKey,
    RequestFailed(String),
    InvalidResponse(String),
    ParseError(String),
}

impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMError::MissingApiKey => write!(f, "API key not found. Set it in ~/.config/mnemosyne/config.toml or use OPENAI_MNEMOSYNE_API_KEY / OPENAI_API_KEY environment variables."),
            LLMError::RequestFailed(e) => write!(f, "Request failed: {}", e),
            LLMError::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            LLMError::ParseError(_) => write!(f, "Could not parse response. Please try again."),
        }
    }
}

impl std::error::Error for LLMError {}

// Provider trait

pub trait LLMProvider {
    fn name(&self) -> &'static str;
    fn send(&self, prompt: &Prompt) -> impl std::future::Future<Output = Result<LLMResponse, LLMError>> + Send;
}

// OpenAI Provider

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

impl OpenAIProvider {
    pub fn new() -> Result<Self, LLMError> {
        let api_key = Config::get_api_key()
            .ok_or(LLMError::MissingApiKey)?;

        Ok(OpenAIProvider {
            client: Client::new(),
            api_key,
            model: String::from("gpt-4o"),
        })
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "OpenAI"
    }

    async fn send(&self, prompt: &Prompt) -> Result<LLMResponse, LLMError> {
        let user_content = format!("{}\n\n---\n\n{}", prompt.instruction, prompt.content);

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: String::from("system"),
                    content: prompt.system.clone(),
                },
                OpenAIMessage {
                    role: String::from("user"),
                    content: user_content,
                },
            ],
            temperature: 0.3,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LLMError::RequestFailed(format!("{}: {}", status, body)));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

        let content = openai_response
            .choices
            .first()
            .ok_or_else(|| LLMError::InvalidResponse("No choices in response".to_string()))?
            .message
            .content
            .clone();

        // NOTE: Clean markdown
        let clean_content = content
            .trim()
            .strip_prefix("```json")
            .or_else(|| content.trim().strip_prefix("```"))
            .and_then(|s| s.strip_suffix("```"))
            .map(|s| s.trim())
            .unwrap_or(content.trim());

        let llm_response: LLMResponse = match serde_json::from_str(clean_content) {
            Ok(resp) => resp,
            Err(e) => {
                log_response(clean_content, Some(&e.to_string()));
                return Err(LLMError::ParseError(format!("{}: {}", e, content)));
            }
        };

        log_response(clean_content, None);
        Ok(llm_response)
    }
}

// Wrappers

pub struct LLMAgent<P: LLMProvider> {
    provider: P,
}

impl<P: LLMProvider> LLMAgent<P> {
    pub fn new(provider: P) -> Self {
        LLMAgent { provider }
    }

    pub async fn send(&self, prompt: &Prompt) -> Result<LLMResponse, LLMError> {
        self.provider.send(prompt).await
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }
}
