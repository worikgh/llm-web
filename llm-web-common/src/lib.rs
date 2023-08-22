//use hmac::{Hmac, Mac};
// use sha2::Sha256;
// use std::error::Error;
// use uuid::Bytes;
// use uuid::Uuid;
// use std::fmt;
pub mod communication;
extern crate llm_rs;
use serde::{Deserialize, Serialize};
// use awasm_bindgen::prelude::*;
// use wasm_bindgen::JsValue;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub uuid: String,
    pub token: String,
}

// #[wasm_bindgen]
// impl SessionInfo {
//     #[wasm_bindgen(constructor)]
//     pub fn new(uuid: String, token: String) -> SessionInfo {
//         SessionInfo { uuid, token }
//     }

//     #[wasm_bindgen(js_name = "toJson")]
//     pub fn to_json(&self) -> JsValue {
//         let json_str = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
//         JsValue::from_str(&json_str)
//     }

//     #[wasm_bindgen(js_name = "fromJson")]
//     pub fn from_json(json: &JsValue) -> String {
//         let session_info_opt: Option<SessionInfo> =
//             serde_json::from_str(&json.as_string().unwrap_or_else(|| "".to_string())).ok();
//         match session_info_opt {
//             Some(session_info) => {
//                 serde_json::to_string(&session_info).unwrap_or_else(|_| "".to_string())
//             }
//             None => "{}".to_string(),
//         }
//     }
// }

// impl fmt::Display for SessionInfo {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Claims{{ sub: {:?}, exp: {:?} }}", self.uuid, self.token)
//     }
// }

// // Function to encode a JWT for a given username and secret
// #[wasm_bindgen]
// pub fn encode_claims(claims: &SessionInfoSessionInfoet: &[u8]) -> Result<String, String> {
//     _encSessionInfoaims(claims, secret).map_err(|e| e.to_string())
// }

// pub fn encode_claims_nowasm(claims: &SessionInfo, secret: &[u8]) -> Result<String, Box<dyn Error>> {
//     _encode_claims(claSessionInfoecret)
// }

// pub fn decode_claims(token: &str, secret: &[u8]) -> Result<SessionInfo, String> {
//     _decode_claims(token, secret).map_err(|e| e.to_string())
// }

// fn _encode_claims(_claims: &SessionInfo, _secret: &[u8]) -> Result<String, Box<dyn Error>> {
//     // let claim_name = usernaSessionInfostring();
//     Ok("Unimplemented".to_string())
// }

// fn _decode_claims(_token: &str, secret:&[u8]) -> Result<SessionInfo, Box<dyn Error>> {
//     let _key: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
//     let claims: SessionInfo = SessionInfo::new("Unimplemented".to_string(), 0);
//     Ok(claims)
// }

// #[wasm_bindgen]
// pub fn timestamp_wts() -> u64 {
//     let timestamp_ms = js_sySessionInfoe::now();
//     (timestamp_ms / 1000.0) as u64
// }
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
