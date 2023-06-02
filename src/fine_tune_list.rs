use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct FineTune {
    pub id: String,
    object: String,
    model: String,
    pub created_at: u64,
    fine_tuned_model: Option<String>,
    hyperparams: HashMap<String, String>,
    organization_id: String,
    result_files: Vec<String>,
    pub status: String,
    validation_files: Vec<String>,
    training_files: Vec<HashMap<String, String>>,
    updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FineTuneList {
    object: String,
    pub data: Vec<FineTune>,
}
