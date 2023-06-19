use llm_web_common::Claims;
use wasm_bindgen::prelude::*;
use web_sys::window;
//use web_sys::console;
use web_sys::{Document, Element};
pub fn set_page(
    f: impl Fn(&Document, &Claims) -> Result<Element, JsValue>,
    claims: &Claims,
) -> Result<(), JsValue> {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let body = document.body().expect("Could not access document.body");
    let e = f(&document, claims)?;
    body.append_child(&e)?;
    Ok(())
}
