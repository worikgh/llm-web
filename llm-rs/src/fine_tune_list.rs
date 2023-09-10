use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct FineTune {
    object: String,
    pub id: String,
    hyperparams: HyperParams,
    organization_id: String,
    model: String,
    training_files: Vec<File>,
    validation_files: Vec<File>,
    result_files: Vec<File>,
    pub created_at: i64,
    updated_at: u64,
    pub status: String,
    fine_tuned_model: Option<String>,
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
struct HyperParams {
    n_epochs: usize,
    batch_size: Option<usize>,
    prompt_loss_weight: f64,
    learning_rate_multiplier: Option<f64>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FineTuneList {
    object: String,
    pub data: Vec<FineTune>,
}

impl FineTuneList {
    pub fn as_string(&self) -> String {
        format!("Fine Tule List: \n{self}")
        // self.data
        //     .iter()
        //     .map(|x| format!("{} {} {}", x.id, x.created_at, x.status))
        //     .collect::<Vec<String>>()
        //     .iter()
        //     .fold(String::new(), |a, b| format!("{a}\t{b}\n"))
    }
}

impl fmt::Display for FineTune {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {}\nModel: {}\nStatus: {}\nCreated At: {}\n",
            self.id, self.model, self.status, self.created_at
        )
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {}\nFilename: {}\nPurpose: {}\nStatus: {}\nCreated At: {}\n",
            self.id,
            self.filename,
            self.purpose,
            self.status,
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(self.created_at, 0).unwrap(),
                Utc,
            )
        )
    }
}

impl fmt::Display for HyperParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "n_epochs: {}\nbatch_size: {:?}\nprompt_loss_weight: {}\nlearning_rate_multiplier: {:?}\n",
            self.n_epochs, self.batch_size, self.prompt_loss_weight, self.learning_rate_multiplier
        )
    }
}

impl fmt::Display for FineTuneList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fine_tunes_str = self
            .data
            .iter()
            .map(|x| {
                format!(
                    "Q {} {} {}",
                    x.id,
                    DateTime::<Utc>::from_naive_utc_and_offset(
                        NaiveDateTime::from_timestamp_opt(x.created_at, 0).unwrap(),
                        Utc,
                    ),
                    x.status
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", fine_tunes_str)
    }
}
