// use chrono::Utc;
// use crate::chat_div::chat_div;
// use crate::set_page::set_page;
// use crate::chat_div::chat_div;
// use crate::make_request::make_request;
use crate::manipulate_css::add_css_rule;
#[allow(unused_imports)]
use crate::set_page::set_page;
use crate::utility::print_to_console;
use crate::utility::print_to_console_s;
#[allow(unused_imports)]
use gloo::{events, timers};
use gloo_events::EventListener;
// use gloo_storage::LocalStorage;
// use gloo_storage::Storage;
#[allow(unused_imports)]
use js_sys::Function;
// use llm_web_common::communication::CommType;
// use llm_web_common::communication::LoginRequest;
// use llm_web_common::communication::LoginResponse;
// use llm_web_common::communication::Message;
use wasm_bindgen::prelude::*;
#[allow(unused_imports)]
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
#[allow(unused_imports)]
use web_sys::{window, Document, Element, EventTarget, HtmlInputElement};

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

    let login_main_div = document.create_element("div")?;
    login_main_div.set_class_name("grid-item");
    login_main_div.set_id("login-div");

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
    login_main_div.append_child(&username_input)?;
    login_main_div.append_child(&password_input)?;
    login_main_div.append_child(&user_text_submit)?;
    grid_container.append_child(&login_main_div)?;
    main_div.append_child(&grid_container)?;

    add_css_rule(document, "html, body", "height", "100%".to_string())?; // Add an event listener to the button.
    add_css_rule(document, "html, body", "margin", "0".to_string())?; // Add an event listener to the button.
    add_css_rule(document, "#login-div > input", "margin", "1em".to_string())?; // Add an event listener to the button.
    add_css_rule(
        document,
        "#login-div",
        "border",
        "2px solid black".to_string(),
    )?; // Add an event listener to the button.
    add_css_rule(document, "#login-div", "width", "50%".to_string())?; // Add an event listener to the button.
    add_css_rule(document, "#login-div", "display", "flex".to_string())?; // Add an event listener to the button.
    add_css_rule(
        document,
        "#login-div",
        "flex-direction",
        "column".to_string(),
    )?; // Add an event listener to the button.
    add_css_rule(document, "#login-div", "padding", "10px".to_string())?; // Add an event listener to the button.
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
        // let login_request = LoginRequest { username, password };
        // let login_message = Message::from(login_request);
        // let cb = |msg: Message| {
        //     match msg.comm_type {
        //         CommType::LoginResponse => {
        //             let lr: LoginResponse = serde_json::from_str(msg.object.as_str()).unwrap();
        //             if lr.success {
        //                 // Store token
        //                 let document = window()
        //                     .and_then(|win| win.document())
        //                     .expect("Failed to get document");

        //                 let head = document.body().unwrap();
        //                 head.set_attribute("data.token", lr.token.unwrap().as_str())
        //                     .unwrap();
        //                 set_page(chat_div).unwrap();
        //             } else {
        //                 // print_to_console("For debugging always goto chat_div");
        //                 // set_page(chat_div).unwrap();
        //                 set_page(login_div).unwrap();
        //             }
        //         }
        //         _ => panic!("Expected LoginResponse got {}", msg),
        //     };
        // };
        // match make_request(login_message, cb) {
        //     Ok(()) => (),
        //     Err(err) => panic!("{:?}", err),
        // };
    });
    on_click.forget();
    Ok(main_div)
}

/// If authenticated return the token.  Else return None
pub fn authenticated() -> Option<String> {
    // let result: Option<String>;
    // match LocalStorage::get::<String>("token") {
    //     Ok(t) => Some(t),
    //     Err(err) => {
    //         eprintln!("eror: {}", err);
    //         None
    //     }
    // }
    None
}

//#[wasm_bindgen]
