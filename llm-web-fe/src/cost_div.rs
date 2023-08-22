/// Display the cost to the user
use web_sys::{Document, Element};

/// Display data about current spending by the user.
pub fn cost_div(document: &Document) -> Element {
    let cost_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    cost_div.set_id("cost-div");
    let cost_text_span = document
        .create_element("span")
        .expect("Could not create DIV element");
    let cost_text_note = document.create_text_node("0.22/0.11/407.83:2");
    cost_text_span.append_child(&cost_text_note).unwrap();
    cost_text_span.set_id("cost-text-span");
    cost_div.append_child(&cost_text_span).unwrap();
    cost_div
}
