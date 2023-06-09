use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    object: String,
    data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
    permission: Vec<ModelPermission>,
    root: String,
    parent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ModelPermission {
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
