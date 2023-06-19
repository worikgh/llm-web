// use js_sys::Function;
/// Interaction with user
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, EventTarget, HtmlInputElement, HtmlTextAreaElement};

/// Handle interaction with a user.  Has mechanism for user to enter
/// text, and a mechanism for the system to respond with a text reply
pub fn interaction_div(document: &Document) -> Result<Element, JsValue> {
    // Create the surrounding DIV
    let interaction_div = document.create_element("div")?;
    interaction_div.set_class_name("grid-item");
    interaction_div.set_id("interaction-div");

    // The DIV to hold the results.  Has a text area
    let results_div = document.create_element("div")?;
    results_div.set_id("results-div");

    let results_text_area = document.create_element("textarea")?;
    results_text_area.set_inner_html("Results here");
    results_text_area.set_id("results-text-area");
    results_div.append_child(&results_text_area)?;

    // The DIV to hold the user's input.  Holds a text entry field and
    // a submit button
    let user_text_input_div = document.create_element("div")?;
    user_text_input_div.set_id("user-text-input-div");
    let user_text_input_input = document.create_element("input")?;
    user_text_input_input.set_id("user-text-input-input");
    user_text_input_input.set_attribute("type", "text")?;
    user_text_input_div.append_child(&user_text_input_input)?;
    let user_text_submit = document.create_element("button")?;
    user_text_submit.set_id("user_text_submit");
    user_text_submit.set_inner_html("Submit");
    user_text_input_div.append_child(&user_text_input_input)?;
    user_text_input_div.append_child(&user_text_submit)?;

    // Assemble the parts
    interaction_div.append_child(&results_div)?;
    interaction_div.append_child(&user_text_input_div)?;

    // Code to process user input
    let submit_button = user_text_submit;
    let input_field = user_text_input_input;
    let results_area = results_text_area;

    // Create a Closure for the submit_button's click event
    let submit_click_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        // Prevent the default behavior of the click (or any?) mouse event
        event.prevent_default();

        // Convert the input_field Element reference into an
        // HtmlInputElement and get the user input
        let input_field: &HtmlInputElement = input_field
            .dyn_ref()
            .expect("Element input_field is not an HtmlInputElement");
        let user_input = input_field.value();

        // Do not do anything much to the input....
        let processed_input = format!("Input is: {user_input}");

        // Convert the results_area Element reference into an HtmlTextAreaElement
        let results_area: &HtmlTextAreaElement = results_area
            .dyn_ref()
            .expect("Element results_area is not an HtmlTextAreaElement");

        results_area.set_value(&processed_input);
    }) as Box<dyn FnMut(_)>)
    .into_js_value()
    .dyn_into::<js_sys::Function>()
    .expect("Closure function failed to cast into JsValue");

    submit_button
        .dyn_ref::<EventTarget>()
        .expect("Element submit_button is not an EventTarget")
        .add_event_listener_with_callback("click", &submit_click_closure)?;

    Ok(interaction_div)
}
