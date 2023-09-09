pub mod communication;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub uuid: String,
    pub token: String,
}
