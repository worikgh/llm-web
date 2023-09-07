use crate::make_request::make_request;
use crate::manipulate_css::add_css_rule;
use crate::set_page::set_focus_on_element;
use crate::utility::print_to_console;
#[allow(unused_imports)]
use crate::utility::print_to_console_s;
use serde::{Deserialize, Serialize};
// use llm_rs;
use llm_web_common::communication::ChatPrompt;
use llm_web_common::communication::ChatResponse;
use llm_web_common::communication::CommType;
use llm_web_common::communication::InvalidRequest;
use llm_web_common::communication::LLMMessage;
use llm_web_common::communication::LLMMessageType;
use llm_web_common::communication::Message;
use wasm_bindgen::prelude::*;
use web_sys::{
    window, Document, Element, HtmlButtonElement, HtmlInputElement, HtmlOptionElement,
    HtmlSelectElement,
};

/// A prompt has returned from the LLM.  Process it here
fn process_chat_response(chat_response: ChatResponse) -> Result<(), JsValue> {
    print_to_console("chat_request 1");
    let mut chat_state = ChatState::restore()?;
    chat_state.responses.push(chat_response);

    // Save the response
    // Get response area
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let result_div = document.get_element_by_id("response-div").unwrap();
    result_div.set_inner_html(chat_state.get_response_display().as_str());
    chat_state.store()?;
    print_to_console("chat_request 2");
    Ok(())
}

/// The callback for `make_request`
fn chat_request(message: Message) {
    print_to_console_s(format!("chat_request 1 {}", message.comm_type));
    match message.comm_type {
        CommType::ChatResponse => {
            print_to_console("chat_request 1.1");
            let chat_response: ChatResponse =
                serde_json::from_str(message.object.as_str()).unwrap();
            process_chat_response(chat_response).unwrap();
            // pub struct ChatRequestInfo {
            // :
            // 	pub model: String,
            // 	pub usage: Usage,
            // 	pub choices: Vec<ChatChoice>,
            // }
            // let model = chat_response.request_info.model;
            // let usage = chat_response.request_info.usage;
            // let choices = chat_response.request_info.choices;
            // let choice = choices.first();
        }
        CommType::InvalidRequest => {
            let inr: InvalidRequest =
                serde_json::from_str(message.object.as_str()).expect("Not an InvalidRequest");
            let document = window()
                .and_then(|win| win.document())
                .expect("Failed to get document");
            print_to_console("chat_request ivr 1");
            let result_div = document.get_element_by_id("response-div").unwrap();
            result_div.set_inner_html(&inr.reason);
        }
        _ => (),
    };
}

/// The callback for the submit button to send a prompt to the model
fn chat_submit() {
    print_to_console("chat_submit 1");
    // Get the contents of the prompt
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let prompt_input: HtmlInputElement = document
        .get_element_by_id("prompt-input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .map_err(|err| format!("Error casting to HtmlInputElement: {:?}", err))
        .unwrap();
    let prompt = prompt_input.value();
    prompt_input.set_value("");
    let model_selection: HtmlSelectElement = document
        .get_element_by_id("model-chat")
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .map_err(|err| format!("Error casting to HtmlOptionsCollection: {err:?}",))
        .unwrap();

    let model: String = if let Some(element) = model_selection.selected_options().item(0) {
        element.get_attribute("value").unwrap()
    } else {
        todo!("Handle this")
    };

    // Get token
    let token: String;
    if let Some(t) = document.body().unwrap().get_attribute("data.token") {
        token = t;
    } else {
        todo!("Set status concerning error: No data token");
    }
    // As a start sending one message and not building a conversation
    let message_role = LLMMessage {
        role: LLMMessageType::System,
        content: "You are a helpful assistant".to_string(),
    };
    let message_body = LLMMessage {
        role: LLMMessageType::User,
        content: prompt,
    };
    let messages = vec![message_role, message_body];
    let chat_prompt = ChatPrompt {
        model,
        messages,
        temperature: 1.0, // Todo: Get this from user interface
        token,
    };

    let message: Message = Message::from(chat_prompt);
    print_to_console("chat_submit 2 submit: calling make_request");
    make_request(message, chat_request).unwrap();
}

/// Maintain the state of the chat dialogue
#[derive(Debug, Deserialize, Serialize)]
struct ChatState {
    responses: Vec<ChatResponse>,
}

impl ChatState {
    /// Store in <data-chat-state> node
    fn store(&self) -> Result<(), JsValue> {
        print_to_console("ChatState::store 1");
        let self_str = serde_json::to_string(self)
            .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()))?;
        let document = window()
            .and_then(|win| win.document())
            .expect("Failed to get document");
        document
            .get_element_by_id("main_body")
            .unwrap()
            .set_attribute("data-chat-div", self_str.as_str())?;
        print_to_console("ChatState::store 1");
        Ok(())
    }

    /// Read a ChatState from <data-chat-state> node
    fn restore() -> Result<Self, JsValue> {
        print_to_console("ChatStatus::restore 1");
        let document = window()
            .and_then(|win| win.document())
            .expect("Failed to get document");
        print_to_console("ChatStatus::restore 2");
        if let Some(e) = document.get_element_by_id("main_body") {
            if let Some(self_str) = e.get_attribute("data-chat-div") {
                return serde_json::from_str(self_str.as_str())
                    .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()));
            }
        }
        Ok(Self {
            responses: Vec::new(),
        })
    }

    /// Get a display to put in response area
    fn get_response_display(&self) -> String {
        let mut result = String::new();
        for i in self.responses.iter() {
            result = format!("{result}<br/>{}", i.response);
        }
        result
    }
}

/// Screen fo the `chat` model interface
pub fn chat_div(document: &Document) -> Result<Element, JsValue> {
    // Manage state
    print_to_console("chat_div 1");

    // The container DIV that arranges the page
    print_to_console("chat_div 2");
    let chat_div = document
        .create_element("div")
        .expect("Could not create DIV element");

    chat_div.set_id("chat-div");
    chat_div.set_class_name("grid-container");

    // The response from LLM
    let response_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    response_div.set_id("response-div");

    // The entry for the prompt
    let prompt_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    prompt_div.set_id("prompt-div");
    let prompt_inp: HtmlInputElement = document
        .create_element("input")
        .map_err(|err| format!("Error creating input element: {:?}", err))?
        .dyn_into::<HtmlInputElement>()
        .map_err(|err| format!("Error casting to HtmlInputElement: {:?}", err))?;
    prompt_inp.set_value("");
    prompt_inp.set_type("text");
    prompt_inp.set_id("prompt-input");
    prompt_div.append_child(&prompt_inp)?;

    // The submit button
    let submit_button: HtmlButtonElement = document
        .create_element("button")
        .map_err(|err| format!("Error creating button element: {:?}", err))?
        .dyn_into::<HtmlButtonElement>()
        .map_err(|err| format!("Error casting to HtmlButtonElement: {:?}", err))?;
    submit_button.set_inner_text("submit");
    submit_button.set_id("chat-submit");
    let closure = Closure::wrap(Box::new(chat_submit) as Box<dyn Fn()>);
    submit_button.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    prompt_div.append_child(&submit_button)?;

    // The side_panel menu
    let side_panel_div = document
        .create_element("div")
        .expect("Could not create DIV element");
    side_panel_div.set_id("side-panel-div");

    // Create the model selection tool
    let select_element = document
        .create_element("select")
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .unwrap();
    select_element.set_id("model-chat");
    let options = select_element.options();

    options.add_with_html_option_element(&HtmlOptionElement::new_with_text_and_value(
        "Gpt-3",
        "gpt-3.5-turbo",
    )?)?;

    options.add_with_html_option_element(&HtmlOptionElement::new_with_text_and_value(
        "Gpt-4", "gpt-4",
    )?)?;
    side_panel_div.append_child(&select_element)?;

    // Put the page together
    chat_div.append_child(&response_div).unwrap();
    chat_div.append_child(&prompt_div).unwrap();
    chat_div.append_child(&side_panel_div).unwrap();

    // Prepare variables to control page layout

    // Column and row count
    let col = 160;
    let row = 100;

    // Arrange Page:

    // * Side Panel takes left part of screen from top of screen down
    // to bottom

    // * The right portion is divided in three, vertically:

    //  At the bottom a prompt entry area and submit button

    //  At the top and taking up most of the remaining space is the
    //  display of results.

    // Side panel starts at top (1) left (1) and its height is the
    // screen heigh. The side panel width (span) is to 4/16 of screen
    // width
    let side_panel_w = col * 4 / 16;
    let side_panel_t = 1;
    let side_panel_l = 1;
    let side_panel_h = row;

    // The response, prompt, and button panels all have the same left
    // margin and width
    let main_l = side_panel_l + side_panel_w;
    let main_w = col - side_panel_w;

    // Prompt div height is 10% of total, start div_height above
    // button_t
    let prompt_h = (row * 10) / 100;
    let prompt_t = row - (row * 10) / 100 + 1;
    let prompt_l = main_l;
    let prompt_w = main_w;

    // Response top is below cost, to the right of the side panel,
    // takes all the space left vertically and extends to the right of
    // the screen
    let response_t = 1;
    let response_l = main_l;
    let response_h = row - prompt_h;
    let response_w = main_w;

    // // Inject the style into the DOM.
    // clear_css(document)?;

    add_css_rule(document, "html, body", "height", "100%".to_string())?;
    add_css_rule(document, "html, body", "margin", "0".to_string())?;
    add_css_rule(document, "html, body", "padding", "0".to_string())?;

    add_css_rule(document, ".grid-container", "display", "grid".to_string())?;
    add_css_rule(document, ".grid-container", "height", "100%".to_string())?;
    add_css_rule(document, ".grid-container", "width", "100%".to_string())?;
    add_css_rule(document, ".grid-container", "padding", "0".to_string())?;
    add_css_rule(document, ".grid-container", "margin", "0".to_string())?;
    add_css_rule(document, ".grid-container", "overflow", "auto".to_string())?;

    add_css_rule(
        document,
        ".grid-container",
        "grid-template-columns",
        format!("repeat({col}, 1fr)"),
    )?;
    add_css_rule(
        document,
        ".grid-container",
        "grid-template-rows",
        format!("repeat({row}, 1fr)"),
    )?;
    add_css_rule(document, ".grid-container", "gap", ".1em".to_string())?;

    add_css_rule(
        document,
        "#response-div",
        "grid-column",
        format!("{response_l} / span {response_w}"),
    )?;
    add_css_rule(
        document,
        "#response-div",
        "grid-row",
        format!("{response_t} / span {response_h}"),
    )?;
    add_css_rule(document, "#response-div", "overflow", "auto")?;

    add_css_rule(
        document,
        "#prompt-div",
        "grid-column",
        format!("{prompt_l} / span {prompt_w}"),
    )?;
    add_css_rule(
        document,
        "#prompt-div",
        "grid-row",
        format!("{prompt_t} / span {prompt_h}"),
    )?;
    add_css_rule(
        document,
        "#prompt-div",
        "border",
        "thick double #ff00ff".to_string(),
    )?;
    add_css_rule(document, "#prompt-div", "display", "flex")?;
    add_css_rule(document, "#prompt-input", "flex-grow", "1")?;

    add_css_rule(
        document,
        "#side-panel-div",
        "grid-column",
        format!("{side_panel_l} / span {side_panel_w}"),
    )?;
    add_css_rule(
        document,
        "#side-panel-div",
        "grid-row",
        format!("{side_panel_t} / span {side_panel_h}"),
    )?;
    add_css_rule(
        document,
        "#side-panel-div",
        "border",
        "thick double #ffff00".to_string(),
    )?;

    // Pad the button to the left
    add_css_rule(document, "#chat-submit", "margin-left", "1em")?;

    set_focus_on_element(document, "prompt-input");

    Ok(chat_div)
}
