mod mock;

pub use mock::MockAgent;

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
