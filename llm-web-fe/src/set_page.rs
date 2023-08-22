#[allow(unused_imports)]
use crate::utility::print_to_console;
#[allow(unused_imports)]
use crate::utility::print_to_console_s;
use wasm_bindgen::prelude::*;
use web_sys::window;
use web_sys::{Document, Element, HtmlElement};
const MAIN_DIV_NAME: &str = "llm-rs";

/// `set_page(f, c)`: Display a page.  `f` is the function that builds
/// the page to display.
pub fn set_page(f: impl Fn(&Document) -> Result<Element, JsValue>) -> Result<(), JsValue> {
    print_to_console("set_page 1");
    // Get the main document
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let body = document.body().expect("Could not access document.body");

    let e = f(&document)?;
    if let Some(old_div) = document.get_element_by_id(MAIN_DIV_NAME) {
        body.remove_child(&old_div)?;
    }

    let new_main_div = document.create_element("DIV").unwrap();
    new_main_div.set_attribute("id", MAIN_DIV_NAME).unwrap();
    new_main_div.append_child(&e)?;
    body.append_child(&new_main_div)?;
    Ok(())
}

pub fn set_focus_on_element(document: &Document, element_id: &str) {
    // let document = web_sys::window().unwrap().document().unwrap();
    if let Some(element) = document.get_element_by_id(element_id) {
        if let Some(input) = element.dyn_ref::<HtmlElement>() {
            input.focus().unwrap();
        } else {
            print_to_console_s(format!(
                "Failed to set focus. Found {element_id} but is not a HtmlElement.  {element:?}"
            ));
        }
    } else {
        print_to_console_s(format!(
            "Failed to set focus.  Could not find: {element_id}"
        ));
    }
}
