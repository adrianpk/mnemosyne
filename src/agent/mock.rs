use super::{Agent, Prompt, Suggestion};

pub struct MockAgent;

impl MockAgent {
    pub fn new() -> Self {
        MockAgent
    }
}

impl Agent for MockAgent {
    fn suggest(&self, prompt: &Prompt) -> Suggestion {
        let words: Vec<&str> = prompt.content.split_whitespace().collect();

        if words.len() < 4 {
            return Suggestion {
                original: prompt.content.clone(),
                replacement: prompt.content.clone(),
                explanation: String::from("Text is too short to analyze."),
            };
        }

        let mut modified = words.clone();

        if words.len() >= 4 {
            modified.swap(1, 3);
        }
        if words.len() >= 8 {
            modified.swap(5, 7);
        }

        Suggestion {
            original: prompt.content.clone(),
            replacement: modified.join(" "),
            explanation: String::from("Swapped a few words for better flow."),
        }
    }
}
