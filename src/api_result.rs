use std::collections::HashMap;

#[derive(Debug)]
pub struct ApiResult {
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ApiResult {
    pub fn new(body: String, headers: HashMap<String, String>) -> Self {
        Self { headers, body }
    }
}
