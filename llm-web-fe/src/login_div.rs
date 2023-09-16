// use chrono::Utc;
// use crate::chat_div::chat_div;
// use crate::set_page::set_page;
// use crate::chat_div::chat_div;

use crate::llm_webpage::LlmWebPage;
use crate::make_request::make_request;
use crate::manipulate_css::add_css_rule;
#[allow(unused_imports)]
use crate::set_page::set_page;
use crate::set_page::set_status;
use crate::set_page::update_cost_display;
use crate::set_page::Pages;
use crate::utility::print_to_console;
use crate::utility::print_to_console_s;
#[allow(unused_imports)]
use gloo::{events, timers};
use gloo_events::EventListener;
#[allow(unused_imports)]
use js_sys::Function;
use llm_web_common::communication::CommType;
use llm_web_common::communication::LoginRequest;
use llm_web_common::communication::LoginResponse;
use llm_web_common::communication::Message;
#[allow(unused_imports)]
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
#[allow(unused_imports)]
use web_sys::{window, Document, Element, EventTarget, HtmlInputElement};
pub struct LoginDiv;
impl LlmWebPage for LoginDiv {
    ///  The login screen
    fn initialise_page(document: &Document) -> Result<Element, JsValue> {
        print_to_console("login_div");

        let login_main_div = document.create_element("div")?;
        login_main_div.set_id("login-main-div");

        let login_fields_div = document.create_element("div")?;
        login_fields_div.set_class_name("grid-item");
        login_fields_div.set_id("login-fields-div");

        // Username and pasword elements
        let username_input = document.create_element("input")?;
        let password_input = document.create_element("input")?;
        username_input.set_id("username_input");
        password_input.set_id("password_input");
        username_input.set_attribute("type", "text")?;
        password_input.set_attribute("type", "password")?;

        // Hack so logging in quicker
        username_input.set_attribute("value", "a")?;
        password_input.set_attribute("value", "b")?;
        // Login button
        let user_text_submit = document.create_element("button")?;
        user_text_submit.set_id("user_text_submit");
        user_text_submit.set_inner_html("Login");

        // Assemble pieces
        login_fields_div.append_child(&username_input)?;
        login_fields_div.append_child(&password_input)?;
        login_fields_div.append_child(&user_text_submit)?;
        login_main_div.append_child(&login_fields_div)?;
        add_css_rule(document, "html, body", "height", "100%".to_string())?;
        add_css_rule(document, "html, body", "margin", "0".to_string())?;
        add_css_rule(
            document,
            "#login-fields-div > input",
            "margin",
            "1em".to_string(),
        )?;
        add_css_rule(
            document,
            "#login-fields-div",
            "border",
            "2px solid black".to_string(),
        )?;
        add_css_rule(document, "#login-fields-div", "width", "50%".to_string())?;
        add_css_rule(document, "#login-fields-div", "display", "flex".to_string())?;
        add_css_rule(
            document,
            "#login-fields-div",
            "flex-direction",
            "column".to_string(),
        )?;
        add_css_rule(document, "#login-fields-div", "padding", "10px".to_string())?;

        let on_click = EventListener::new(&user_text_submit, "click", move |_event| {
            // Call whatever function you would like with the value of the
            // text input.
            let username: String = if let Some(input) = username_input.dyn_ref::<HtmlInputElement>()
            {
                input.value()
            } else {
                "".to_string()
            };
            let password: String = if let Some(input) = password_input.dyn_ref::<HtmlInputElement>()
            {
                input.value()
            } else {
                "".to_string()
            };

            print_to_console_s(format!("click! username: {username} {:?}", username_input));
            let login_request = LoginRequest { username, password };
            let login_message = Message::from(login_request);
            let cb = |msg: Message| {
                print_to_console_s(format!("login_div call back.  message: {msg:?}"));
                match msg.comm_type {
                    CommType::LoginResponse => {
                        let lr: LoginResponse = serde_json::from_str(msg.object.as_str()).unwrap();
                        let document = window()
                            .and_then(|win| win.document())
                            .expect("Failed to get document");
                        if lr.success {
                            // Store token
                            let token = lr.token.unwrap();
                            set_status(&document, format!("Setting token: {token}").as_str());

                            let head = document.body().unwrap();
                            head.set_attribute("data.token", token.as_str()).unwrap();
                            set_page(Pages::ChatDiv).unwrap();
                            update_cost_display(&document, lr.credit, 0.0, 0.0);
                        } else {
                            set_status(&document, "Login failed");
                            set_page(Pages::LoginDiv).unwrap();
                        }
                    }
                    _ => panic!("Expected LoginResponse got {}", msg),
                };
            };
            match make_request(login_message, cb) {
                Ok(()) => (),
                Err(err) => panic!("{:?}", err),
            };
        });
        on_click.forget();
        Ok(login_main_div)
    }
}
