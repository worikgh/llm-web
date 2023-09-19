/// Make a XmlHttpRequest to the backend.  
#[allow(unused_imports)]
use crate::utility::{print_to_console, print_to_console_s};
use llm_web_common::communication::{CommType, Message};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::XmlHttpRequest;
/// `message` is the data to send to back end
/// `f` is the function to call with the response.  It will call `set_page`

pub fn make_request(
    message: Message,
    mut callback_onload: impl FnMut(Message) + 'static,
    callback_onabort: impl FnMut() + 'static,
) -> Result<XmlHttpRequest, JsValue> {
    // print_to_console_s(format!("make_request 1"));
    let api = match message.comm_type {
        CommType::LoginRequest => "login",
        CommType::ChatPrompt => "chat",
        _ => {
            print_to_console_s(format!("make_request Unimplemented: {message}"));
            panic!("Unimplemented")
        }
    };
    let uri = format!("/api/{api}");
    let xhr: XmlHttpRequest = XmlHttpRequest::new().unwrap();
    xhr.open("POST", uri.as_str())?;
    xhr.set_request_header("Content-Type", "application/json")?;
    let xhr_clone = xhr.clone();
    let cb = Closure::wrap(Box::new(move |_data: JsValue| {
        if xhr_clone.ready_state() == 4 && xhr_clone.status().unwrap() == 200 {
            let response = xhr_clone.response_text().unwrap().unwrap();
            // Do something with response..
            let message: Message = serde_json::from_str(response.as_str()).unwrap();
            callback_onload(message);
        }
    }) as Box<dyn FnMut(_)>);

    xhr.set_onload(Some(cb.as_ref().unchecked_ref()));
    cb.forget();
    let callback_onabort = Rc::new(RefCell::new(callback_onabort));

    let callback_onabort_clone = callback_onabort.clone();
    let closure_onabort = Closure::wrap(Box::new(move || {
        (*callback_onabort_clone.borrow_mut())();
    }) as Box<dyn Fn()>);
    xhr.set_onabort(Some(closure_onabort.as_ref().unchecked_ref()));
    closure_onabort.forget();
    let message_str = serde_json::to_string(&message).unwrap();
    xhr.send_with_opt_u8_array(Some(message_str.as_str().as_bytes()))
        .unwrap();

    Ok(xhr)
}
