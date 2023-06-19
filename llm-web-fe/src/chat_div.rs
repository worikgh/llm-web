use crate::cost_div::cost_div;
/// Control a chat model
use crate::interaction_div::interaction_div;
use llm_web_common::Claims;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};
/// Screen fo the `chat` model interface
pub fn chat_div(document: &Document, claims: &Claims) -> Result<Element, JsValue> {
    let main_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    main_div.set_id("main-div");
    let grid_container = document
        .create_element("div")
        .expect("Could not create DIV element");
    grid_container.set_class_name("grid-container");

    let cost_div = cost_div(document);
    let interaction_div = interaction_div(document)?;
    cost_div.set_class_name("grid-item");
    cost_div.set_id("cost-display");

    grid_container.append_child(&cost_div).unwrap();
    grid_container.append_child(&interaction_div).unwrap();

    main_div.append_child(&grid_container).unwrap();
    Ok(main_div)
}
