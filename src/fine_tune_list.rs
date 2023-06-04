use serde::{Deserialize, Serialize};

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
    pub created_at: u64,
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
    created_at: u64,
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
        self.data
            .iter()
            .map(|x| format!("{} {} {}", x.id, x.created_at, x.status))
            .collect::<Vec<String>>()
            .iter()
            .fold(String::new(), |a, b| format!("{a}\t{b}\n"))
    }
}
