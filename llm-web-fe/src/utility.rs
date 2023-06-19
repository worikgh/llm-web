// Utility functons for llm-web-fe

use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn print_to_console(message: &str) {
    console::log_1(&message.into());
}
#[wasm_bindgen]
pub fn print_to_console_s(message: String) {
    console::log_1(&message.into());
}
