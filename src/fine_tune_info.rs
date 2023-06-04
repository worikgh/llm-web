extern crate chrono;
use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FineTuneInfo {
    object: String,
    id: String,
    hyperparams: HyperParams,
    organization_id: String,
    model: String,
    training_files: Vec<File>,
    validation_files: Vec<File>,
    result_files: Vec<File>,
    created_at: i64,
    updated_at: u64,
    status: String,
    fine_tuned_model: String,
    events: Vec<FineTuneEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HyperParams {
    n_epochs: u32,
    batch_size: u32,
    prompt_loss_weight: f64,
    learning_rate_multiplier: f64,
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
struct FineTuneEvent {
    object: String,
    level: String,
    message: String,
    created_at: i64,
}

impl FineTuneInfo {
    pub fn as_string(&self) -> String {
        let id = &self.id;
        let model = &self.model;
        let params: String = format!(
            "{}/{}/{}/{}",
            self.hyperparams.n_epochs,
            self.hyperparams.batch_size,
            self.hyperparams.prompt_loss_weight,
            self.hyperparams.learning_rate_multiplier
        );
        let events: String = self.events.iter().fold("".to_string(), |a, b| {
            let ts = DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(b.created_at, 0).unwrap(),
                Utc,
            )
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string();
            let level: String = if b.level != "info" {
                b.level.clone()
            } else {
                "".to_string()
            };
            let message = b.message.clone();
            format!("{a}\n\t{ts}: {message}. {level}")
        });
        format!("{id}: {model} {params}: {events}")
    }
}

// let dt: DateTime<Utc> = DateTime::from_utc(chrono::NaiveDateTime::from_timestamp(unix_timestamp, 0), Utc);
// let formatted_dt: String = dt.format("%Y-%m-%d %H:%M:%S %Z").to_string();
