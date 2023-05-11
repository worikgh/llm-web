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
impl ApiResult<Vec<(String, String)>> {
    pub fn new_v(body: Vec<(String, String)>, headers: HashMap<String, String>) -> Self {
        Self { headers, body }
    }
}
impl ApiResult<()> {
    pub fn new_e(headers: HashMap<String, String>) -> Self {
        Self { headers, body: () }
    }
}
