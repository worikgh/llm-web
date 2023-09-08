// use chat_div::chat_div;
use login_div::login_div;
use set_page::initialise_page;
use set_page::set_page;
use wasm_bindgen::prelude::*;
// use web_sys::{window, HtmlLinkElement};
#[allow(unused_imports)]
use crate::utility::print_to_console;
mod chat_div;
mod cost_div;
mod filters;
mod login_div;
mod make_request;
mod manipulate_css;
mod set_page;
mod utility;
// Called when the wasm module is instantiated
// #[wasm_bindgen(start)]
// fn main() -> Result<(), JsValue> {
//     // Use `web_sys`'s global `window` function to get a handle on the global
//     // window object.
//     let window = web_sys::window().expect("no global `window` exists");
//     let document = window.document().expect("should have a document on window");
//     let body = document.body().expect("document should have a body");

//     // Manufacture the element we're gonna append
//     let val = document.create_element("p")?;
//     val.set_inner_html("llm-web-fe");

//     body.append_child(&val)?;

//     Ok(())
// }
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    initialise_page()?;
    start_app()
}

fn start_app() -> Result<(), JsValue> {
    set_page(login_div)?;
    Ok(())
}
