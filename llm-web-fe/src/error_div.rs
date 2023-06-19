/// Control a chat model
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

/// Screen fo the `chat` model interface
pub fn error_div(document: &Document) -> Result<Element, JsValue> {
    let main_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    main_div.set_id("main-div");
    let grid_container = document
        .create_element("div")
        .expect("Could not create DIV element");
    grid_container.set_class_name("grid-container");

    let inner_div = inner_div(document);
    inner_div.set_class_name("grid-item");
    inner_div.set_id("cost-display");

    grid_container.append_child(&inner_div).unwrap();

    main_div.append_child(&grid_container).unwrap();
    Ok(main_div)
}

fn inner_div(document: &Document) -> Element {
    let inner_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    let text_span = document
        .create_element("span")
        .expect("Could not create DIV element");
    let note = document.create_text_node("There was an error");
    text_span.append_child(&note).unwrap();
    text_span.set_id("error-text-span");
    inner_div.append_child(&text_span).unwrap();
    inner_div
}
