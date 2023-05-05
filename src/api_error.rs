use reqwest::StatusCode;
use std::collections::HashMap;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ApiErrorType {
    BadJson(String),
    FailedRequest(String),
    Error(String),
    Status(StatusCode),
}

#[derive(Debug)]
pub struct ApiError {
    pub error_type: ApiErrorType,
    pub headers: HashMap<String, String>,
}
impl ApiError {
    pub fn new(error_type: ApiErrorType, headers: HashMap<String, String>) -> Self {
        Self {
            error_type,
            headers,
        }
    }
}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header_report: String = self
            .headers
            .iter()
            .fold(String::new(), |a, (b0, b1)| format!("{a}\n{b0}:{b1}"));
        match self.error_type {
            ApiErrorType::FailedRequest(ref msg) => {
                write!(f, "{header_report}Failed Request: {msg}")
            }
            ApiErrorType::BadJson(ref msg) => write!(f, "Bad JSON: {}", msg),

            // HTTP failure.  Not a 200 status
            ApiErrorType::Status(ref status) => {
                write!(f, "{header_report}HTTP Status({status}")
            }

            // Generic.  TODO: Get rid of this
            ApiErrorType::Error(ref msg) => write!(f, "{header_report}Error: {msg}"),
        }
    }
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
