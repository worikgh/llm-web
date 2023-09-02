/// Make a XmlHttpRequest to the backend.  
use crate::utility::print_to_console_s;
use llm_web_common::communication::{CommType, Message};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::XmlHttpRequest;
/// `message` is the data to send to back end
/// `f` is the function to call with the response.  It will call `set_page`
pub fn make_request(message: Message, mut f: impl FnMut(Message) + 'static) -> Result<(), JsValue> {
    let api = match message.comm_type {
        CommType::LoginRequest => "login",
        CommType::ChatPrompt => "chat",
        _ => {
            print_to_console_s(format!("make_request Unimplemented: {message}"));
            panic!("Unimplemented")
        }
    };
    let uri = format!("/api/{api}");
    let xhr = XmlHttpRequest::new().unwrap();
    print_to_console_s(format!("make_request({message}). 1"));
    xhr.open_with_async("POST", uri.as_str(), true)?;
    xhr.set_request_header("Content-Type", "application/json")?;

    let xhr = Rc::new(RefCell::new(xhr));
    let xhr_clone = xhr.clone();

    let onreadystatechange_callback = Closure::wrap(Box::new(move || {
        let xhr = xhr_clone.borrow();
        if xhr.ready_state() == 4 && xhr.status().unwrap() == 200 {
            let response = xhr.response_text().unwrap().unwrap();
            // Do something with response..
            print_to_console_s(format!("State == 4: Response: {response}"));
            let message: Message = serde_json::from_str(response.as_str()).unwrap();
            f(message);
        }
    }) as Box<dyn FnMut()>);

    xhr.borrow_mut()
        .set_onreadystatechange(Some(onreadystatechange_callback.as_ref().unchecked_ref()));

    // Save the closure to avoid it being dropped
    onreadystatechange_callback.forget();

    let message_str = serde_json::to_string(&message).unwrap();
    xhr.borrow()
        .send_with_opt_u8_array(Some(message_str.as_str().as_bytes()))
        .unwrap();

    Ok(())
}
