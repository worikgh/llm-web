// use chrono::Utc;
// use crate::chat_div::chat_div;
// use crate::set_page::set_page;
use crate::chat_div::chat_div;
#[allow(unused_imports)]
use crate::set_page::set_page;
use crate::utility::print_to_console;
use crate::utility::print_to_console_s;

#[allow(unused_imports)]
use gloo::{events, timers};
use gloo_events::EventListener;
use gloo_storage::LocalStorage;
use gloo_storage::Storage;
#[allow(unused_imports)]
use js_sys::Function;
use llm_web_common::communication::LoginRequest;
use llm_web_common::communication::Message;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::XmlHttpRequest;
#[allow(unused_imports)]
use web_sys::{Document, Element, EventTarget, HtmlInputElement};

///  The login screen
#[wasm_bindgen]
pub fn login_div(document: &Document) -> Result<Element, JsValue> {
    print_to_console("login_div");
    let main_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    main_div.set_id("main-div");

    let grid_container = document
        .create_element("div")
        .expect("Could not create DIV element");
    grid_container.set_class_name("grid-container");
    main_div.append_child(&grid_container)?;

    let login_div = document.create_element("div")?;
    login_div.set_class_name("grid-item");
    login_div.set_id("login-div");

    // Username and pasword elements
    let username_input = document.create_element("input")?;
    let password_input = document.create_element("input")?;
    username_input.set_id("username_input");
    password_input.set_id("password_input");
    username_input.set_attribute("type", "text")?;
    password_input.set_attribute("type", "password")?;

    // Login button
    let user_text_submit = document.create_element("button")?;
    user_text_submit.set_id("user_text_submit");
    user_text_submit.set_inner_html("Login");

    // Assemble pieces
    login_div.append_child(&username_input)?;
    login_div.append_child(&password_input)?;
    login_div.append_child(&user_text_submit)?;
    grid_container.append_child(&login_div)?;
    main_div.append_child(&grid_container)?;

    // Add an event listener to the button.
    let on_click = EventListener::new(&user_text_submit, "click", move |_event| {
        // Call whatever function you would like with the value of the
        // text input.
        let username: String = if let Some(input) = username_input.dyn_ref::<HtmlInputElement>() {
            input.value()
        } else {
            "".to_string()
        };
        let password: String = if let Some(input) = password_input.dyn_ref::<HtmlInputElement>() {
            input.value()
        } else {
            "".to_string()
        };

        print_to_console_s(format!("click! username: {username} {:?}", username_input));
        let login_request = LoginRequest { username, password };
        let login_message = Message::from(login_request);
        let message = serde_json::to_string(&login_message).unwrap();
        match make_request(message.as_str()) {
            Ok(()) => (),
            Err(err) => panic!("{:?}", err),
        };
    });
    on_click.forget();
    Ok(main_div)
}

/// If authenticated return the token.  Else return None
pub fn authenticated() -> Option<String> {
    let result: Option<String>;
    match LocalStorage::get::<String>("token") {
        Ok(t) => Some(t),
        Err(err) => {
            eprintln!("eror: {}", err);
            None
        }
    }
}

#[wasm_bindgen]
pub fn make_request(message: &str) -> Result<(), JsValue> {
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
            set_page(chat_div).unwrap();
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
