use rsa::RsaPublicKey;
/// Structures to send back and forth between llm-web-fe and llm-web-be utilises
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
/// The communication between the front end and the back end uses
/// `Message` struct.  `CommType` categorises the communication and
/// defines what object is being relayed in the `Message.object` type
pub enum CommType {
    LoginRequest,
    LoginResponse,
    ChatPrompt,
}

/// The messae as sent: `comm_type` says what type it is, the String
/// is encoded JSON of the object itself.  The `user` field identifies
/// which frontend is communicating.  There is one front end per user.
/// TODO: What happens if the same user logs in twice?  The secong
/// login should overwrite the first(?)
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub comm_type: CommType,
    pub object: String,
    pub user: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
// From llm-web-fe -> llm-web-be.  A prompt for a chat session
pub struct ChatPrompt {
    /// The model to use    
    pub model: String,

    /// The conversation this prompt is part of.  This lets be and fe
    /// synchronise themselves
    pub chat_id: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub payload: Vec<u8>,
    pub public_key: RsaPublicKey,
}
