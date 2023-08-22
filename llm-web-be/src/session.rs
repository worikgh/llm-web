/// User Session
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Session {
    pub uuid: Uuid,            // Id user
    pub expire: DateTime<Utc>, // When session expires
    pub token: String,         // Encrypted token user must pass to use session
}
