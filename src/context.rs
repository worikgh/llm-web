/// The context of a GPT Chat
use serde::{Deserialize, Serialize};
use std::mem;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Context {
    pub purpose: String,
    pub cost: f64, // IN cents, and fraction of a cent
    pub prompt_response: Vec<String>,
}

impl Context {
    pub fn new(purpose: &str) -> Context {
        Context {
            purpose: purpose.to_string(),
            cost: 0.0,
            prompt_response: Vec::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.prompt_response.len()
    }
    pub fn is_empty(&self) -> bool {
        self.prompt_response.is_empty()
    }
    pub fn as_string(&self) -> String {
        let purpose = &self.purpose;
        let mut exchange = String::new();
        for i in 0..self.prompt_response.len() {
            let next_bit = format!(
                "\n\t{}: {}",
                if i % 2 == 0 { "user" } else { "assistant" },
                self.prompt_response[i]
            );
            exchange += next_bit.as_str();
        }
        format!("Purpose: {purpose}{exchange}\n")
    }
    pub fn push(&mut self, s: String) {
        self.prompt_response.push(s);
    }
    pub fn clear(&mut self) {
        self.prompt_response.clear();
        self.cost = 0.0;
    }
    pub fn sz(&self) -> usize {
        // Memory usage of the purpose String
        let purpose_size = mem::size_of_val(self.purpose.as_str());

        // Memory usage of the prompt_response Vec itself
        let vec_size = mem::size_of_val(&self.prompt_response);

        // Memory usage of the Strings inside the prompt_response Vec
        let mut strs_size = 0;
        for s in &self.prompt_response {
            strs_size += mem::size_of_val(s.as_str());
        }

        // Total memory usage
        purpose_size + vec_size + strs_size
    }
}
