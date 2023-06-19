#![allow(unused_variables)]
use llm_web_common::Claims;
use rand::prelude::*;
use wasm_bindgen::prelude::*;
mod chat_div;
mod claims;
mod cost_div;
/// A frontend to Large Language Models (LLMs).  The backend is
/// supplied by [`llm-rs`](https://crates.io/crates/llm-rs)
mod interaction_div;
mod login_div;
mod set_page;
mod utility;
use chat_div::chat_div;
use login_div::authenticated;
use login_div::login_div;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use set_page::set_page;
use utility::print_to_console;
use web_sys::window;

use crate::utility::print_to_console_s;
/// The main entry point.
fn start_app() -> Result<(), JsValue> {
    print_to_console("start_app");

    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    // Encrypt
    let data = b"hello world";
    let enc_data = pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &data[..])
        .expect("failed to encrypt");
    assert_ne!(&data[..], &enc_data[..]);

    // Decrypt
    let dec_data = priv_key
        .decrypt(Pkcs1v15Encrypt, &enc_data)
        .expect("failed to decrypt");
    assert_eq!(&data[..], &dec_data[..]);

    print_to_console_s(format!("start_app: {:?}", pub_key));

    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");

    let body = document.body().expect("Could not access document.body");

    if let Some(claims) = authenticated() {
        set_page(chat_div, &claims)?;
    } else {
        set_page(login_div, &Claims::new("".to_string(), 0))?;
    }

    Ok(())
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    start_app()
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
#[wasm_bindgen]
pub fn subtract(a: u32, b: u32) -> u32 {
    a - b
}
#[wasm_bindgen]
pub fn concat(a: u32, b: u32) -> String {
    format!("Two numbers! {a} {b}")
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// And finally, we don't even have to define the `log` function ourselves! The
// `web_sys` crate already has it defined for us.
