use crate::utility::print_to_console_s;
use llm_web_common::communication::Message;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::XmlHttpRequest;
pub fn make_request(message: &str, mut f: impl FnMut(Message) + 'static) -> Result<(), JsValue> {
    let xhr = XmlHttpRequest::new().unwrap();
    print_to_console_s(format!("make_request({message}). 1"));
    xhr.open_with_async("POST", "/api/login", true)?;
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
            // set_page(chat_div).unwrap();
            // console::log_1(&JsValue::from_str(&response));
        }
    }) as Box<dyn FnMut()>);

    xhr.borrow_mut()
        .set_onreadystatechange(Some(onreadystatechange_callback.as_ref().unchecked_ref()));

    // Don't forget to save your closure to avoid it being dropped
    onreadystatechange_callback.forget();

    xhr.borrow()
        .send_with_opt_u8_array(Some(message.as_bytes()))
        .unwrap();

    Ok(())
}
