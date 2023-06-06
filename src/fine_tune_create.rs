use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Serialize, Deserialize, Debug)]
struct FineTuneEvent {
    object: String,
    level: String,
    message: String,
    created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    object: String,
    id: String,
    purpose: String,
    filename: String,
    bytes: u64,
    created_at: i64,
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
    created_at: i64,
    updated_at: u64,
    status: String,
    fine_tuned_model: Option<String>,
    events: Vec<FineTuneEvent>,
}

impl Display for FineTuneEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}, {}, {})", self.object, self.level, self.message)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "({}: {})", self.object, self.filename)
    }
}

impl Display for FineTuneCreate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}: model={}, status={}, training_files=[{:#?}], validation_files=[{:#?}], result_files=[{:#?}]",
            self.id,
            self.model,
            self.status,
            self.training_files,
            self.validation_files,
            self.result_files
        )
    }
}
