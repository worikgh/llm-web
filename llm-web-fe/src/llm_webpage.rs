use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};
pub trait LlmWebPage {
    fn initialise_page(document: &Document) -> Result<Element, JsValue>;
}
