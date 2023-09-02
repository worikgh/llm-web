// use llm_rs;
/// Structures to send back and forth between llm-web-fe and llm-web-be utilises
//use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};
use std::fmt;
//use uuid::serde;
use uuid::Uuid;
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
/// The communication between the front end and the back end uses
/// `Message` struct.  `CommType` categorises the communication and
/// defines what object is being relayed in the `Message.object` type
pub enum CommType {
    LoginRequest,
    LoginResponse,
    LogoutRequest,
    LogoutResponse,
    ChatPrompt,
    ChatResponse,
    InvalidRequest,
}

/// The messae as sent: `comm_type` says what type it is, the String
/// is encoded JSON of the object itself.  
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub comm_type: CommType,
    pub object: String,
}

/// Server -> Client.  
#[derive(Debug, Deserialize, Serialize)]
pub struct InvalidRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub uuid: Option<Uuid>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutRequest {
    pub uuid: Uuid,
    pub token: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct LogoutResponse {
    pub success: bool, // Will only fail if not logged in (FLW)
}

#[derive(Debug, Deserialize, Serialize)]
// From llm-web-fe -> llm-web-be.  A prompt for a chat session
pub struct ChatPrompt {
    /// The model to use    
    pub model: String,

    // The prompt being sent
    pub prompt: String,

    // The user's authenticating data
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// From llm-web-be -> llm-web-fe.  Response from LLM
pub struct ChatResponse {
    pub request_info: String,
}
// Display for CommType
impl fmt::Display for CommType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommType::LoginRequest => write!(f, "Login Request"),
            CommType::LoginResponse => write!(f, "Login Response"),
            CommType::LogoutRequest => write!(f, "Logout Request"),
            CommType::LogoutResponse => write!(f, "Logout Response"),
            CommType::ChatPrompt => write!(f, "Chat Prompt"),
            CommType::ChatResponse => write!(f, "Chat Response"),
            CommType::InvalidRequest => write!(f, "Invalid Request"),
        }
    }
}

// Display for Message
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "comm_type: {}, object: {}", self.comm_type, self.object)
    }
}

// Display for LoginRequest
impl fmt::Display for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "username: {}, password: {}",
            self.username, self.password
        )
    }
}

// Display for ChatPrompt
impl fmt::Display for ChatPrompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "model: {}", self.model)
    }
}

// Display for LoginResponse
impl fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Success: {} ", self.success,)
    }
}

// Write a `from` method for every struct that is not `Message`
// converting them to a `Message`.  Use `serde_json::to_string` to
// initialise the `object` fields.
impl From<InvalidRequest> for Message {
    fn from(request: InvalidRequest) -> Self {
        Message {
            comm_type: CommType::InvalidRequest,
            object: serde_json::to_string(&request).unwrap(),
        }
    }
}

impl From<LoginRequest> for Message {
    fn from(request: LoginRequest) -> Self {
        Message {
            comm_type: CommType::LoginRequest,
            object: serde_json::to_string(&request).unwrap(),
        }
    }
}

impl From<ChatPrompt> for Message {
    fn from(prompt: ChatPrompt) -> Self {
        Message {
            comm_type: CommType::ChatPrompt,
            object: serde_json::to_string(&prompt).unwrap(),
        }
    }
}

impl From<LoginResponse> for Message {
    fn from(response: LoginResponse) -> Self {
        Message {
            comm_type: CommType::LoginResponse,
            object: serde_json::to_string(&response).unwrap(),
        }
    }
}

impl From<LogoutRequest> for Message {
    fn from(request: LogoutRequest) -> Self {
        Message {
            comm_type: CommType::LogoutRequest,
            object: serde_json::to_string(&request).unwrap(),
        }
    }
}
impl From<LogoutResponse> for Message {
    fn from(response: LogoutResponse) -> Self {
        Message {
            comm_type: CommType::LogoutResponse,
            object: serde_json::to_string(&response).unwrap(),
        }
    }
}
