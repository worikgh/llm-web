extern crate chrono;
use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Serialize, Deserialize)]
pub struct FineTuneRetrieve {
    object: String,
    id: String,
    hyperparams: Hyperparams,
    organization_id: String,
    model: String,
    training_files: Vec<File>,
    validation_files: Vec<File>,
    result_files: Vec<File>,
    created_at: i64,
    updated_at: i64,
    status: String,
    fine_tuned_model: String,
    events: Vec<FineTuneEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hyperparams {
    n_epochs: u32,
    batch_size: u32,
    prompt_loss_weight: f64,
    learning_rate_multiplier: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    object: String,
    id: String,
    purpose: String,
    filename: String,
    bytes: u64,
    created_at: i64,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_details: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FineTuneEvent {
    object: String,
    level: String,
    message: String,
    created_at: i64,
}

impl FineTuneRetrieve {
    pub fn as_string(&self) -> String {
        format!("Fine Tune: {self}")
    }
}

impl Display for FineTuneRetrieve {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Object: {}", self.object)?;
        writeln!(f, "ID: {}", self.id)?;
        writeln!(f, "Organization ID: {}", self.organization_id)?;
        writeln!(f, "Model: {}", self.model)?;
        writeln!(f, "Status: {}", self.status)?;
        writeln!(f, "Fine Tuned Model: {}", self.fine_tuned_model)?;

        writeln!(f, "Hyperparameters:")?;
        writeln!(f, "  N Epochs: {}", self.hyperparams.n_epochs)?;
        writeln!(f, "  Batch Size: {}", self.hyperparams.batch_size)?;
        writeln!(
            f,
            "  Prompt Loss Weight: {}",
            self.hyperparams.prompt_loss_weight
        )?;
        writeln!(
            f,
            "  Learning Rate Multiplier: {}",
            self.hyperparams.learning_rate_multiplier
        )?;

        writeln!(f, "\nTraining Files:")?;
        for file in &self.training_files {
            writeln!(f, "  {}", file.filename)?;
        }

        writeln!(f, "\nValidation Files:")?;
        for file in &self.validation_files {
            writeln!(f, "  {}", file.filename)?;
        }

        writeln!(f, "\nResult Files:")?;
        for file in &self.result_files {
            writeln!(f, "  {}", file.filename)?;
        }

        writeln!(f, "\nEvents:")?;
        for event in &self.events {
            let ts = DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(event.created_at, 0).unwrap(),
                Utc,
            )
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string();
            writeln!(f, "  {ts}: {}: {}", event.level, event.message)?;
        }

        Ok(())
    }
}
