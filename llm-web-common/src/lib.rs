use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::error::Error;
use std::fmt;
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub token: Option<String>,
}
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

/// Implement the JWT Claims
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    pub exp: u64,
}

#[wasm_bindgen]
impl Claims {
    #[wasm_bindgen(getter)]
    pub fn sub(&self) -> String {
        self.sub.clone()
    }
    #[wasm_bindgen(setter)]
    pub fn set_sub(&mut self, sub: String) {
        self.sub = sub;
    }
    // This function constructs a new Claims instance from JS
    #[wasm_bindgen(constructor)]
    pub fn new(sub: String, exp: u64) -> Claims {
        Claims { sub, exp }
    }
}

impl fmt::Display for Claims {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Claims{{ sub: {}, exp: {} }}", self.sub, self.exp)
    }
}

// Function to encode a JWT for a given username and secret
#[wasm_bindgen]
pub fn encode_claims(claims: &Claims, secret: &[u8]) -> Result<String, String> {
    _encode_claims(claims, secret).map_err(|e| e.to_string())
}

pub fn encode_claims_nowasm(claims: &Claims, secret: &[u8]) -> Result<String, Box<dyn Error>> {
    _encode_claims(claims, secret)
}

pub fn decode_claims(token: &str, secret: &[u8]) -> Result<Claims, String> {
    _decode_claims(token, secret).map_err(|e| e.to_string())
}

fn _encode_claims(claims: &Claims, secret: &[u8]) -> Result<String, Box<dyn Error>> {
    // let claim_name = username.to_string();
    Ok("Unimplemented".to_string())
}

fn _decode_claims(token: &str, secret: &[u8]) -> Result<Claims, Box<dyn Error>> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
    let claims: Claims = Claims::new("Unimplemented".to_string(), 0);
    Ok(claims)
}

#[wasm_bindgen]
pub fn timestamp_wts() -> u64 {
    let timestamp_ms = js_sys::Date::now();
    (timestamp_ms / 1000.0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}