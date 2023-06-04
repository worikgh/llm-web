use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct FineTuneEvent {
    object: String,
    level: String,
    message: String,
    created_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    object: String,
    id: String,
    purpose: String,
    filename: String,
    bytes: u64,
    created_at: u64,
    status: String,
    status_details: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FineTuneCreate {
    object: String,
    id: String,
    hyperparams: HashMap<String, Option<f64>>,
    organization_id: String,
    model: String,
    training_files: Vec<File>,
    validation_files: Vec<File>,
    result_files: Vec<File>,
    created_at: u64,
    updated_at: u64,
    status: String,
    fine_tuned_model: Option<String>,
    events: Vec<FineTuneEvent>,
}
