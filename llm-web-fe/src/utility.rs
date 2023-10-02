// Utility functons for llm-web-fe

use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn _print_to_console(message: &str) {
    console::log_1(&message.into());
}

pub fn print_to_console<T: Into<String>>(message: T) {
    let message = message.into();
    _print_to_console(message.as_str());
}
