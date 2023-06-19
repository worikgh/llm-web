// use chrono::Utc;
use crate::chat_div::chat_div;
use crate::set_page::set_page;
use crate::utility::print_to_console;
use crate::utility::print_to_console_s;
use gloo_storage::LocalStorage;
use gloo_storage::Storage;

use llm_web_common::decode_claims;
use llm_web_common::encode_claims;
use llm_web_common::timestamp_wts;
use llm_web_common::Claims;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, EventTarget, HtmlInputElement};
const SECRET_KEY: [u8; 20] = *b"example_secret_keys!"; // The secret key to sign your JWTs

///  The login screen
#[wasm_bindgen]
pub fn login_div(document: &Document, claims: &Claims) -> Result<Element, JsValue> {
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

    let user_name_input = document.create_element("input")?;
    user_name_input.set_id("user_name_input");
    user_name_input.set_attribute("type", "text")?;

    let password_input = document.create_element("input")?;
    password_input.set_id("password_input");
    password_input.set_attribute("type", "password")?;

    let user_text_submit = document.create_element("button")?;
    user_text_submit.set_id("user_text_submit");
    user_text_submit.set_inner_html("Login");

    login_div.append_child(&user_name_input)?;
    login_div.append_child(&password_input)?;
    login_div.append_child(&user_text_submit)?;
    grid_container.append_child(&login_div)?;
    main_div.append_child(&grid_container)?;
    // Code to process user input
    let submit_button = user_text_submit;
    let name_field = user_name_input;
    let password_field = password_input;

    // Create a Closure for the submit_button's click event
    let submit_click_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        // Prevent the default behavior of the click (or any?) mouse event
        event.prevent_default();

        // Convert the input_field Element reference into an
        // HtmlInputElement and get the user input
        let name_field: &HtmlInputElement = name_field
            .dyn_ref()
            .expect("Element input_field is not an HtmlInputElement");
        let username = name_field.value();

        let password_field: &HtmlInputElement = password_field
            .dyn_ref()
            .expect("Element input_field is not an HtmlInputElement");
        let password = password_field.value();
        match login(username.as_str(), password.as_str()) {
            Ok(t) => {
                // Logged in.  Put `t` in the environment
                match LocalStorage::set("token", &t) {
                    Ok(_) => match set_page(chat_div, &t) {
                        Ok(()) => (),
                        Err(err) => {
                            print_to_console_s(format!("{:?}: Failed to set token: {:?}", err, t))
                        }
                    },
                    Err(err) => print_to_console_s(format!("{err}: Failed to set token: {:?}", t)),
                }
            }
            Err(err) => print_to_console_s(format!("Failed to login: {err}")),
        };
    }) as Box<dyn FnMut(_)>)
    .into_js_value()
    .dyn_into::<js_sys::Function>()
    .expect("Closure function failed to cast into JsValue");

    submit_button
        .dyn_ref::<EventTarget>()
        .expect("Element submit_button is not an EventTarget")
        .add_event_listener_with_callback("click", &submit_click_closure)?;
    Ok(main_div)
}

// Login function that takes a username and a password, returns a JWT
// if the credentials are valid
pub fn login(username: &str, password: &str) -> Result<Claims, String> {
    // Fetch the user's actual password from your database
    let user_password = "password";

    if password == user_password {
        let claims = Claims::new(username.to_string(), timestamp_wts());
        match encode_claims(&claims, &SECRET_KEY) {
            Ok(jwt) => {
                let claims: Claims = decode_claims(jwt.as_str(), &SECRET_KEY)?;
                Ok(claims)
            }
            Err(_) => Err("Error creating JWT.".to_string()),
        }
    } else {
        Err("Invalid credentials.".to_string())
    }
}

/// If authenticated return the token.  Else return None
pub fn authenticated() -> Option<Claims> {
    let result: Option<Claims>;
    match LocalStorage::get::<Claims>("token") {
        Ok(claims) => {
            if claims.exp > timestamp_wts() {
                result = Some(claims);
            } else {
                result = None;
            }
        }
        Err(err) => {
            result = None;
        }
    };

    result
}

/// The secret that we use for encodingkey and decoding JWT
pub fn get_secret() -> &'static [u8] {
    return &SECRET_KEY;
}
