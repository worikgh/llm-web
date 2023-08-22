#![allow(unused_variables)]
use wasm_bindgen::prelude::*;
mod chat_div;
mod claims;
mod cost_div;
/// A frontend to Large Language Models (LLMs).  The backend is
/// supplied by [`llm-rs`](https://crates.io/crates/llm-rs)
mod login_div;
mod make_request;
mod manipulate_css;
mod set_page;
extern crate llm_rs;
mod utility;
use chat_div::chat_div;
use login_div::authenticated;
use login_div::login_div;
use set_page::set_page;
#[allow(unused_imports)]
use utility::print_to_console;
use web_sys::{window, HtmlLinkElement};

#[allow(unused_imports)]
use crate::utility::print_to_console_s;
/// The main entry point.

/// Main wasm entry point. Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    start_app()
}

fn start_app() -> Result<(), JsValue> {
    let rng = rand::thread_rng();

    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");

    // Style
    let link: HtmlLinkElement = document.create_element("link").unwrap().dyn_into().unwrap();
    link.set_rel("stylesheet");
    link.set_href("/style.css");
    document.head().unwrap().append_child(&link).unwrap();

    let body = document.body().expect("Could not access document.body");

    if let Some(claims) = authenticated() {
        set_page(chat_div)?;
    } else {
        set_page(login_div)?;
    }

    Ok(())
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
