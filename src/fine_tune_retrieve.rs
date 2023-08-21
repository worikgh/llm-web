use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Deserialize)]
struct FineTuneEvent {
    object: String,
    level: String,
    message: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct File {
    object: String,
    id: String,
    purpose: Option<String>,
    filename: String,
    bytes: i64,
    created_at: i64,
    status: String,
    status_details: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HyperParameters {
    n_epochs: u32,
    batch_size: Option<u32>,
    prompt_loss_weight: f64,
    learning_rate_multiplier: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct FineTuneRetrieve {
    object: String,
    id: String,
    hyperparams: HyperParameters,
    organization_id: String,
    model: String,
    training_files: Vec<File>,
    validation_files: Vec<File>,
    result_files: Vec<File>,
    created_at: i64,
    updated_at: i64,
    status: String,
    fine_tuned_model: Option<String>,
    events: Vec<FineTuneEvent>,
}

impl Display for FineTuneRetrieve {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(
            f,
            "object: {}\nid: {}\norganization_id: {}\nmodel: {}\ncreated_at: {}\nupdated_at: {}\nstatus: {}",
            self.object, self.id, self.organization_id, self.model,
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(self.created_at, 0).unwrap(),
                Utc,
            ),
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(self.updated_at, 0).unwrap(),
                Utc,
            ), self.status
        )?;

        writeln!(f, "Hyper Parameters:")?;
        writeln!(f, "  n_epochs: {}", self.hyperparams.n_epochs)?;
        if let Some(batch_size) = self.hyperparams.batch_size {
            writeln!(f, "  batch_size: {}", batch_size)?;
        }
        writeln!(
            f,
            "  prompt_loss_weight: {}",
            self.hyperparams.prompt_loss_weight
        )?;
        if let Some(lr_multiplier) = self.hyperparams.learning_rate_multiplier {
            writeln!(f, "  learning_rate_multiplier: {}", lr_multiplier)?;
        }

        if let Some(ref fine_tuned_model) = self.fine_tuned_model {
            writeln!(f, "fine_tuned_model: {}", fine_tuned_model)?;
        }

        writeln!(f, "Training Files:")?;
        for file in &self.training_files {
            writeln!(f, "  - {}", file.filename)?;
        }

        writeln!(f, "Validation Files:")?;
        for file in &self.validation_files {
            writeln!(f, "  - {}", file.filename)?;
        }

        writeln!(f, "Result Files:")?;
        for file in &self.result_files {
            writeln!(f, "  - {}", file.filename)?;
        }

        writeln!(f, "Events:")?;
        for event in &self.events {
            writeln!(
                f,
                "  - [{}] {}: {} at {}",
                event.level,
                event.object,
                event.message,
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(event.created_at, 0).unwrap(),
                    Utc,
                )
            )?;
        }

        Ok(())
    }
}
