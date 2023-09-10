//! Webapp front end for accessing large language models.
#[allow(unused_imports)]
use crate::utility::print_to_console;
use login_div::login_div;
use set_page::initialise_page;
use set_page::set_page;
use wasm_bindgen::prelude::*;
mod chat_div;
mod cost_div;
mod filters;
mod login_div;
mod make_request;
mod manipulate_css;
mod set_page;
mod utility;
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    initialise_page()?;
    start_app()
}

fn start_app() -> Result<(), JsValue> {
    set_page(login_div)?;
    Ok(())
}
