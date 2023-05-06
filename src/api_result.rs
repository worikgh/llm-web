use std::collections::HashMap;

#[derive(Debug)]
pub struct ApiResult<T> {
    pub headers: HashMap<String, String>,
    pub body: T,
}

impl ApiResult<String> {
    pub fn new(body: String, headers: HashMap<String, String>) -> Self {
        Self { headers, body }
    }
}
