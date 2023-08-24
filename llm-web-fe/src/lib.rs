// use chat_div::chat_div;
use login_div::login_div;
use set_page::set_page;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlLinkElement};
// mod chat_div;
mod login_div;
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
    start_app()
}

fn start_app() -> Result<(), JsValue> {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");

    // Style
    let link: HtmlLinkElement = document.create_element("link").unwrap().dyn_into().unwrap();
    link.set_rel("stylesheet");
    link.set_href("/style.css");
    document.head().unwrap().append_child(&link).unwrap();

    let body = document.body().expect("Could not access document.body");

    set_page(login_div)?;

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
