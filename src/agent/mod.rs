mod mock;
pub mod llm;
pub mod prompts;

pub use mock::MockAgent;
pub use llm::{LLMAgent, LLMError, LLMProvider, OpenAIProvider};

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a field that can be either a string or an array of strings.
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::String(s) if s.is_empty() => Ok(Vec::new()),
        StringOrVec::String(s) => Ok(vec![s]),
        StringOrVec::Vec(v) => Ok(v),
    }
}

// LLM Response Contract 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseMode {
    Replace,
    Suggest,
    Critique,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultBlock {
    pub mode: ResponseMode,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notes {
    pub language_detected: String,
    pub contains_quoted_text: bool,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub result: ResultBlock,
    #[serde(default, deserialize_with = "string_or_vec")]
    pub alternatives: Vec<String>,
    #[serde(default, deserialize_with = "string_or_vec")]
    pub comments: Vec<String>,
    pub notes: Notes,
}

impl LLMResponse {
    /// Convert to legacy Suggestion for UI compatibility
    pub fn to_suggestion(&self, original: &str) -> Suggestion {
        Suggestion {
            original: original.to_string(),
            replacement: self.result.text.clone(),
            explanation: self.comments.join("\n"),
        }
    }
}

// Prompt structures

pub struct Prompt {
    pub system: String,
    pub instruction: String,
    pub content: String,
}

impl Prompt {
    pub fn new(system: &str, instruction: &str, content: &str) -> Self {
        Prompt {
            system: system.to_string(),
            instruction: instruction.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Suggestion {
    pub original: String,
    pub replacement: String,
    pub explanation: String,
}

pub trait Agent {
    fn suggest(&self, prompt: &Prompt) -> Suggestion;
}
