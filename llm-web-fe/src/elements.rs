//! Create an HTMLElement.  Reduce boilerplate by putting it here
//! behind functions
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlDivElement, HtmlInputElement, HtmlSpanElement};

pub fn create_div(document: &Document, id: Option<&str>) -> Result<HtmlDivElement, JsValue> {
    let result = document
        .create_element("div")
        .map_err(|err| format!("Error creating DIV for Role editing : {:?}", err))?
        .dyn_into::<HtmlDivElement>()
        .map_err(|err| format!("Error casting to HtmlLabelElement: {:?}", err))?;
    if let Some(id) = id {
        result.set_id(id);
    }
    Ok(result)
}

pub fn create_input(document: &Document, id: Option<&str>) -> Result<HtmlInputElement, JsValue> {
    let result = document
        .create_element("input")
        .map_err(|err| format!("Error creating button element: {:?}", err))?
        .dyn_into::<HtmlInputElement>()
        .map_err(|err| format!("Error casting to HtmlImageElement: {:?}", err))?;
    if let Some(id) = id {
        result.set_id(id);
    }
    Ok(result)
}

pub fn create_span(document: &Document, id: Option<&str>) -> Result<HtmlSpanElement, JsValue> {
    let result = document
        .create_element("span")
        .map_err(|err| format!("Error creating span element: {:?}", err))?
        .dyn_into::<HtmlSpanElement>()
        .map_err(|err| format!("Error casting to HtmlSpanElement: {:?}", err))?;
    if let Some(id) = id {
        result.set_id(id);
    }
    Ok(result)
}
