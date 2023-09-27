use crate::chat_div::ChatDiv;
use crate::filters::text_for_html;
use crate::llm_webpage::LlmWebPage;
use crate::login_div::LoginDiv;
use crate::manipulate_css::add_css_rule;
#[allow(unused_imports)]
use crate::utility::{print_to_console, print_to_console_s};
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlButtonElement};
use web_sys::{Document, HtmlElement};

pub enum Pages {
    ChatDiv,
    LoginDiv,
}
/// `set_page(f)`: Display a page.  `f` is the function that builds
/// the page to display.
/// Structure of the page
pub fn set_page(page: Pages) -> Result<(), JsValue> {
    // Get the main document
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let body = document.body().expect("Could not access document.body");

    let e = match page {
        Pages::ChatDiv => ChatDiv::initialise_page(&document)?,
        Pages::LoginDiv => LoginDiv::initialise_page(&document)?,
    };
    if let Some(main_body) = document.get_element_by_id("main_body") {
        main_body.set_inner_html("");
        main_body.append_child(&e)?;
        body.append_child(&main_body)?;
        set_focus_on_element(&document, "prompt-input");
    } else {
        print_to_console("No `main_body` in page.  Has not been initialised");
        panic!("Died");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn set_status(status: &str) {
    let document: &Document = &get_doc();
    let status = &text_for_html(status);
    if let Some(status_element) = document.get_element_by_id("status_div") {
        status_element.set_inner_html(status);
    } else {
        print_to_console_s(format!("Status (No status-div): {status}"));
    }
}

#[allow(dead_code)]
pub fn set_focus_on_element(document: &Document, element_id: &str) {
    // print_to_console("set_focus_on_element 1");
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

/// Set up the basic page with header, footer, and body
#[allow(dead_code)]
pub fn initialise_page() -> Result<(), JsValue> {
    // print_to_console("initialise_page()");
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let body = document.body().expect("Could not access document.body");
    while let Some(child) = body.first_child() {
        let _ = body.remove_child(&child);
    }

    // Set up the three divs
    let footer_div = document.create_element("div")?;
    footer_div.set_id("footer");
    let header_div = document.create_element("div")?;
    header_div.set_id("header");
    let main_body = document.create_element("div")?;
    main_body.set_id("main_body");

    // Add a cost area to display cost
    let cost_div = document.create_element("div")?;
    cost_div.set_id("cost_div");
    header_div.append_child(&cost_div)?;

    // Add a status area
    let status_div = document.create_element("div")?;
    status_div.set_id("status_div");
    footer_div.append_child(&status_div)?;
    // Add the divs
    body.append_child(&footer_div)?;
    body.append_child(&header_div)?;
    body.append_child(&main_body)?;

    // The style.  Sizes given in integer units of percent
    let footer_height = 10;
    let header_height = 10;
    let main_body_height = 100 - (footer_height + header_height);
    let main_width = 100;
    add_css_rule(&document, "html, body", "height", "100%")?;
    add_css_rule(&document, "html, body", "margin", "0")?;
    add_css_rule(&document, "html, body", "padding", "0")?;
    add_css_rule(&document, "#header", "height", format!("{header_height}%"))?;
    add_css_rule(&document, "#header", "width", format!("{main_width}%"))?;
    add_css_rule(&document, "#header", "position", "fixed")?;
    add_css_rule(&document, "#header", "top", "0")?;
    add_css_rule(&document, "#header", "left", "0")?;
    add_css_rule(&document, "#sidepanel", "height", "100%")?;
    add_css_rule(&document, "#sidepanel", "width", "0%")?;
    add_css_rule(&document, "#sidepanel", "position", "fixed")?;
    add_css_rule(&document, "#sidepanel", "top", "0")?;
    add_css_rule(&document, "#sidepanel", "left", "0")?;
    add_css_rule(&document, "#footer", "height", format!("{footer_height}%"))?;
    add_css_rule(&document, "#footer", "width", format!("{main_width}%"))?;
    add_css_rule(&document, "#footer", "position", "fixed")?;
    add_css_rule(
        &document,
        "#footer",
        "top",
        format!("{}%", 100 - footer_height),
    )?;
    add_css_rule(&document, "#footer", "left", "0%")?;

    add_css_rule(
        &document,
        "#main_body",
        "height",
        format!("{main_body_height}%"),
    )?;
    add_css_rule(&document, "#main_body", "width", format!("{main_width}%"))?;
    add_css_rule(&document, "#main_body", "position", "fixed")?;
    add_css_rule(&document, "#main_body", "top", format!("{header_height}%"))?;
    add_css_rule(&document, "#main_body", "left", "0%")?;
    add_css_rule(
        &document,
        "#main_body",
        "bottom",
        format!("{}%", 100 - footer_height),
    )?;
    add_css_rule(&document, "#sidepanel", "border", "1px solid black")?;
    add_css_rule(&document, "#footer", "border", "1px solid black")?;
    add_css_rule(&document, "#header", "border", "1px solid black")?;
    add_css_rule(&document, "#main_body", "border", "1px solid black")?;

    add_css_rule(&document, "#cost_div", "float", "right")?;
    add_css_rule(&document, "#cost_div", "background-color", "#f2fbfa")?;

    Ok(())
}

/// Update the cost display
pub fn update_cost_display(document: &Document, credit: f64, this_cost: f64) {
    let cost_div = document.get_element_by_id("cost_div").unwrap();
    let cost_string = format!("Last Prompt: {this_cost:.4}\u{00A2} Credit: {credit:.2}\u{00A2}");
    cost_div.set_inner_html(cost_string.as_str());
}

/// Make a button
pub fn new_button(
    document: &Document,
    id: &str,
    display: &str,
) -> Result<HtmlButtonElement, JsValue> {
    // print_to_console("new_button 1");
    let result: HtmlButtonElement = document
        .create_element("button")
        .map_err(|err| format!("Error creating button element: {:?}", err))?
        .dyn_into::<HtmlButtonElement>()
        .map_err(|err| format!("Error casting to HtmlButtonElement: {:?}", err))?;

    result.set_id(id);
    result.set_inner_text(display);

    Ok(result)
}

pub fn get_doc() -> Document {
    window()
        .and_then(|win| win.document())
        .expect("Failed to get document")
}
