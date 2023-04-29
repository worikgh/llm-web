//! The structures for building the Json prompts
//

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Choice {
    // Field that are present in the response but that are not used here
    // logprobs: Option<Vec<f32>>,
    // index: i32,
    pub text: String,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Response for a completions request.  See
/// https://platform.openai.com/docs/api-reference/completions/create
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequestInfo {
    // The `id` is in response but not used here
    // #[serde(skip_serializing)]
    // id: String,
    #[serde(skip_serializing)]
    pub object: String,
    #[serde(skip_serializing)]
    pub choices: Vec<Choice>,
    #[serde(skip_deserializing)]
    pub prompt: String,
    pub model: String,
    #[serde(skip_deserializing)]
    temperature: f32,
    #[serde(skip_deserializing)]
    max_tokens: u32,
    #[serde(skip_serializing)]
    pub usage: Usage,
}

/// Response for a chats request.  See
/// https://platform.openai.com/docs/api-reference/chat/create

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
#[derive(Serialize, Debug, Deserialize)]
#[serde(tag = "t")]
pub struct ChatChoice {
    index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageURL {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageRequestInfo {
    created: u64,
    pub data: Vec<ImageURL>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequestInfo {
    id: String,
    pub object: String,
    created: u64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<ChatChoice>,
}

/// To receive the transcribed text
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioTranscriptionResponse {
    pub text: String,
}

impl CompletionRequestInfo {
    pub fn new(prompt: &str, model: &str, temperature: f32, max_tokens: u32) -> Self {
        Self {
            choices: Vec::new(),
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            // id: String::new(),
            object: String::new(),
            prompt: prompt.to_string(),
            model: model.to_string(),
            temperature,
            max_tokens,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Permission {
    id: String,
    object: String,
    created: u64,
    allow_create_engine: bool,
    allow_sampling: bool,
    allow_logprobs: bool,
    allow_search_indices: bool,
    allow_view: bool,
    allow_fine_tuning: bool,
    organization: String,
    group: Option<String>,
    is_blocking: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
    permission: Vec<Permission>,
    pub root: String,
    parent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelReturned {
    object: String,
    pub data: Vec<Model>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelExampleData;

    #[test]
    fn models_from_json() {
        let model_data = ModelExampleData::new();
        let json_data: String = model_data.json;
        let v: ModelReturned = match serde_json::from_str(json_data.as_str()) {
            Ok(v) => v,
            Err(err) => panic!("{err}"),
        };
        eprintln!("{:?}", &v);
        assert!(!v.data.is_empty());
    }
}

/// Response for a "models" query
#[derive(Debug, Serialize, Deserialize)]
struct ModelData {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelRequestInfo {
    data: Vec<ModelData>,
}
