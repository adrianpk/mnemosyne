mod mock;
pub mod llm;
pub mod prompts;

pub use mock::MockAgent;
pub use llm::{LLMAgent, LLMError, LLMProvider, OpenAIProvider};

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a field that can be a string, array of strings, or array of objects.
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::String(s) if s.is_empty() => Ok(Vec::new()),
        Value::String(s) => Ok(vec![s]),
        Value::Array(arr) => {
            let strings: Vec<String> = arr
                .into_iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s),
                    Value::Object(obj) => {
                        // Handle objects like {"expression": "...", "explanation": "..."}
                        let expr = obj.get("expression").and_then(|v| v.as_str()).unwrap_or("");
                        let expl = obj.get("explanation").and_then(|v| v.as_str()).unwrap_or("");
                        if !expr.is_empty() && !expl.is_empty() {
                            Some(format!("'{}' — {}", expr, expl))
                        } else if !expl.is_empty() {
                            Some(expl.to_string())
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect();
            Ok(strings)
        }
        Value::Null => Ok(Vec::new()),
        _ => Ok(Vec::new()),
    }
}

/// Deserialize a boolean that might come as string "true"/"false" or actual bool.
fn bool_or_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        String(String),
    }

    match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(b) => Ok(b),
        BoolOrString::String(s) => Ok(s.to_lowercase() == "true"),
    }
}

// LLM Response Contract 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseMode {
    Replace,
    Suggest,
    Critique,
    None,
}

/// Deserialize confidence that might come in various cases.
fn deserialize_confidence<'de, D>(deserializer: D) -> Result<Confidence, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "low" => Ok(Confidence::Low),
        "medium" => Ok(Confidence::Medium),
        "high" => Ok(Confidence::High),
        _ => Ok(Confidence::Medium), // Default to medium for unknown values
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
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
    #[serde(deserialize_with = "bool_or_string")]
    pub contains_quoted_text: bool,
    #[serde(deserialize_with = "deserialize_confidence")]
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
