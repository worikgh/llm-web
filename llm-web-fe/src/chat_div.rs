use crate::filters::filter_html;
use crate::llm_webpage::LlmWebPage;
use crate::make_request::make_request;
use crate::manipulate_css::add_css_rule;
use crate::manipulate_css::clear_css;
use crate::manipulate_css::get_css_rules;
use crate::manipulate_css::set_css_rules;
use crate::set_page::set_focus_on_element;
use crate::set_page::set_status;
use crate::set_page::update_cost_display;
use crate::utility::print_to_console;
#[allow(unused_imports)]
use crate::utility::print_to_console_s;
//use gloo::dialogs::prompt;
use gloo_events::EventListener;
use llm_web_common::communication::ChatPrompt;
use llm_web_common::communication::ChatResponse;
use llm_web_common::communication::CommType;
use llm_web_common::communication::InvalidRequest;
use llm_web_common::communication::LLMMessage;
use llm_web_common::communication::LLMMessageType;
use llm_web_common::communication::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use web_sys::KeyboardEvent;

use wasm_bindgen::prelude::*;
use web_sys::{
    window, Document, Element, HtmlButtonElement, HtmlInputElement, HtmlOptionElement,
    HtmlSelectElement,
};

/// A conversation.  If `prompt` is not `None` a chat prompt has been
/// sent and a reply is being waited for
#[derive(Debug, Deserialize, Serialize)]
struct Conversation {
    prompt: Option<String>,
    responses: Vec<(String, ChatResponse)>,
}

impl Conversation {
    fn new() -> Self {
        Self {
            prompt: None,
            responses: Vec::new(),
        }
    }
    /// Get a display to put in response area.  Transform the text
    /// into HTML, and put class definitions in for prompts and
    /// responses so they can be styled
    fn get_response_display(&self) -> String {
        let mut result = String::new();
        for i in self.responses.iter() {
            let prompt = i.0.as_str();
            let prompt = filter_html(prompt);
            let respone = i.1.response.as_str();
            let respone = filter_html(respone);
            result = format!("{result}<br/><span class='prompt'>{prompt}</span><br/><span class='response'>{respone}</span>",);
        }
        result
    }
}

#[derive(Debug, Deserialize, Serialize)]
/// All the conversations in play
pub struct Chats {
    conversations: HashMap<usize, Conversation>,
    // `current_conversation` is `None` when there is no current
    // conversation, at the beginning and when user clears a
    // conversation.
    current_conversation: Option<usize>,
    credit: f64,
}

impl Chats {
    fn new() -> Result<Self, JsValue> {
        let document = window()
            .and_then(|win| win.document())
            .expect("Failed to get document");
        if let Some(main_body) = document.get_element_by_id("main_body") {
            if let Some(self_str) = main_body.get_attribute("data-chat-div") {
                // There is data in teh DOM to make a `Chats` object from.  Use it
                return serde_json::from_str(self_str.as_str())
                    .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()));
            }
        }
        // There is no existing `Chats` object serialised in the DOM
        // so create a new one
        Ok(Self {
            conversations: HashMap::new(),
            current_conversation: None,
            credit: 0.0, // TODO:  Fix this!
        })
    }

    // The current conversation is where the focus of the user is. It
    // must be:
    // * initialised when a conversation starts .
    // * updated when a response received
    // * References to read it
    // * Reference to mutate it
    fn initialise_current_conversation(&mut self) {
        // Generate a index for the conversation.  This will ensure
        // there are usize::MAX conversations, ever, during the life
        // time of this interface
        let index = match self.conversations.keys().len() {
            0 => 0,
            _ => {
                let indexes = self.conversations.keys().collect::<Vec<&usize>>();
                // TODO There has to be a better way to get the maximum index already
                indexes.iter().fold(0, |a, b| if **b > a { **b } else { a })
            }
        };
        self.conversations.insert(index, Conversation::new());
        self.current_conversation = Some(index);
    }

    /// Update conversation when response received
    fn update_current_conversation(&mut self, response: ChatResponse) -> Result<(), JsValue> {
        // Preconditions:
        // 1. There is a current conversation
        // 2. The `prompt` is not None in current conversation
        print_to_console("update_current_conversation 1 ");
        let conversation = self.get_current_conversation_mut().unwrap();
        print_to_console_s(format!(
            "update_current_conversation 1.1 conversation: {conversation:?}"
        ));
        let prompt: String = conversation.prompt.as_ref().unwrap().clone();
        print_to_console("update_current_conversation 1.2 ");
        conversation.prompt = None;
        conversation.responses.push((prompt, response));
        print_to_console("update_current_conversation 2");
        Ok(())
    }

    /// Get the current conversation to read
    fn get_current_conversation(&self) -> Option<&Conversation> {
        if let Some(cv) = &self.current_conversation {
            self.conversations.get(cv)
        } else {
            None
        }
    }

    /// Get the current conversation to mutate
    fn get_current_conversation_mut(&mut self) -> Option<&mut Conversation> {
        if let Some(cv) = &mut self.current_conversation {
            self.conversations.get_mut(cv)
        } else {
            None
        }
    }
}

impl Drop for Chats {
    /// When `Chats` goes out of scope and is destructed ensure that
    /// its data is saved in the DOM
    fn drop(&mut self) {
        let document = window()
            .and_then(|win| win.document())
            .expect("Failed to get document");
        let main_body = document
            .get_element_by_id("main_body")
            .expect("Must be a #main_body");
        let s: String = serde_json::to_string(self).expect("Failed to encode self as JSON");
        main_body
            .set_attribute("data-chat-div", &s)
            .expect("Failed to save self to DOM");
    }
}

/// Hold the code for creating and manipulating the chat_div
#[derive(Debug, Deserialize)]
pub struct ChatDiv;

impl LlmWebPage for ChatDiv {
    /// Screen for the `chat` model interface
    fn initialise_page(document: &Document) -> Result<Element, JsValue> {
        // Manage state of the conversations with the LLM
        let chats = Arc::new(Mutex::new(Chats::new()?));

        // The container DIV that arranges the page
        let chat_div = document
            .create_element("div")
            .expect("Could not create DIV element");

        chat_div.set_id("chat_div");
        chat_div.set_class_name("grid-container");

        // The conversation with the LLM
        let conversation_div = document
            .create_element("div")
            .expect("Could not create DIV element");
        conversation_div.set_id("response_div");

        // The entry for the prompt
        let prompt_div = document
            .create_element("div")
            .expect("Could not create DIV element");
        prompt_div.set_id("prompt_div");
        let prompt_inp: HtmlInputElement = document
            .create_element("input")
            .map_err(|err| format!("Error creating input element: {:?}", err))?
            .dyn_into::<HtmlInputElement>()
            .map_err(|err| format!("Error casting to HtmlInputElement: {:?}", err))?;
        prompt_inp.set_value("");
        prompt_inp.set_type("text");
        prompt_inp.set_id("prompt_input");
        let cc = chats.clone();

        // Detect when an <enter> key pressed and submit prompt
        let prompt_input_enter = EventListener::new(&prompt_inp, "keyup", move |event| {
            let c = cc.clone();
            let event: KeyboardEvent = event.clone().unchecked_into();
            let key_code = event.key_code();
            if key_code == 13 {
                // <enter> keycode
                chat_submit_cb(c);
            }
        });
        prompt_input_enter.forget();

        prompt_div.append_child(&prompt_inp)?;

        // The submit button
        let submit_button: HtmlButtonElement = document
            .create_element("button")
            .map_err(|err| format!("Error creating button element: {:?}", err))?
            .dyn_into::<HtmlButtonElement>()
            .map_err(|err| format!("Error casting to HtmlButtonElement: {:?}", err))?;
        submit_button.set_inner_text("submit");
        submit_button.set_id("chat-submit");
        let cc = chats.clone();
        let closure = Closure::wrap(Box::new(move || chat_submit_cb(cc.clone())) as Box<dyn Fn()>);
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

        // The clear response button
        // Make a button that clears the responses
        let clear_response = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        clear_response.set_inner_text("Clear Conversation");
        clear_response.set_id("clear_response");
        let ss = chats.clone();
        let clear_conversation_closure = Closure::wrap(Box::new(move || {
            let document = window()
                .and_then(|win| win.document())
                .expect("Failed to get document");
            let result_div = document.get_element_by_id("response_div").unwrap();
            result_div.set_inner_html("");
            let s = ss.clone();
            if let Ok(mut c) = s.lock() {
                c.current_conversation = None;
                let credit = c.credit;
                update_cost_display(&document, credit, 0.0, 0.0);
            };
        }) as Box<dyn Fn()>);
        clear_response.set_onclick(Some(clear_conversation_closure.as_ref().unchecked_ref()));
        clear_conversation_closure.forget();
        side_panel_div.append_child(&clear_response)?;

        // Experimental button
        let clear_style = document
            .create_element("button")
            .unwrap()
            .dyn_into::<HtmlButtonElement>()
            .unwrap();
        clear_style.set_inner_text("Style Experiment");
        clear_style.set_id("clear_style");
        let resp_closure = Closure::wrap(Box::new(|| {
            print_to_console("resp_closure 1");
            let document = window()
                .and_then(|win| win.document())
                .expect("Failed to get document");
            let mut cs_rules = get_css_rules(&document).unwrap();
            cs_rules
                .insert("#side-panel-div", "background-color", "aliceblue")
                .unwrap();
            clear_css(&document).unwrap();
            print_to_console("resp_closure 2");

            set_css_rules(&document, &cs_rules).unwrap();
            print_to_console("resp_closure 3");
        }) as Box<dyn Fn()>);
        clear_style.set_onclick(Some(resp_closure.as_ref().unchecked_ref()));
        resp_closure.forget();
        side_panel_div.append_child(&clear_style)?;

        // Put the page together
        chat_div.append_child(&conversation_div).unwrap();
        chat_div.append_child(&prompt_div).unwrap();
        chat_div.append_child(&side_panel_div).unwrap();

        // Prepare variables to control page layout

        // Column and row count
        let col = 160;
        let row = 100;

        // Arrange Page:

        // * Side Panel takes left part of screen from under the menu
        // bar to the top of status

        // * The right portion is divided in two, vertically:

        //  At the bottom a prompt entry area and submit button

        //  At the top and taking up most of the remaining space is the
        //  display of results.

        // Side panel starts at top (1) left (1) and its height is the
        // screen heigh. The side panel width (span) is to 4/16 of screen
        // width
        let side_panel_w = col * 4 / 16;
        let side_panel_l = 1;
        let side_panel_t = 1;
        let side_panel_h = row;

        // The response, prompt, and button panels all have the same left
        // margin and width
        let main_l = side_panel_l + side_panel_w;
        let main_w = col - side_panel_w;

        // Prompt div height is 10% of total
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

        add_css_rule(document, ".prompt", "font-size", "small")?;
        add_css_rule(document, ".prompt", "color", "#e86d6d")?;
        add_css_rule(document, ".prompt", "background-color", "#fff4f4")?;

        add_css_rule(document, ".response", "font-size", "small")?;
        add_css_rule(document, ".response", "color", "#450627")?;
        add_css_rule(document, ".response", "background-color", "#f3f2f2")?;

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
            "#response_div",
            "grid-column",
            format!("{response_l} / span {response_w}"),
        )?;
        add_css_rule(
            document,
            "#response_div",
            "grid-row",
            format!("{response_t} / span {response_h}"),
        )?;
        add_css_rule(document, "#response_div", "overflow-y", "scroll")?;
        add_css_rule(document, "#response_div", "overflow-wrap", "break-word")?;

        add_css_rule(
            document,
            "#prompt_div",
            "grid-column",
            format!("{prompt_l} / span {prompt_w}"),
        )?;
        add_css_rule(
            document,
            "#prompt_div",
            "grid-row",
            format!("{prompt_t} / span {prompt_h}"),
        )?;
        add_css_rule(
            document,
            "#prompt_div",
            "border",
            "thick double #ff00ff".to_string(),
        )?;
        add_css_rule(document, "#prompt_div", "display", "flex")?;
        add_css_rule(document, "#prompt_input", "flex-grow", "1")?;

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

        set_focus_on_element(document, "prompt_input");

        Ok(chat_div)
    }
}

/// Called to construct the messages for a request.  Each interaction
/// with the LLM includes a history of prevous interactions.  In the
/// general case this is the history of the current conversation.
/// `prompt` is the user's latest input
fn build_messages(chats: Arc<Mutex<Chats>>, prompt: String) -> Vec<LLMMessage> {
    // `messages` is the historical response, build it here.
    let mut result: Vec<LLMMessage> = Vec::new();

    // The "role" is first.  Allways using the same role (TODO: this
    // needs to be configurable)
    result.push(LLMMessage {
        role: LLMMessageType::System,
        content: "You are a helpful assistant".to_string(),
    });

    let mut chats = chats.lock().unwrap();
    // Then the history of the conversation
    match (*chats).get_current_conversation() {
        Some(conversation) => {
            for i in 0..conversation.responses.len() {
                // chat_state.responses[i] has a prompt and a response.
                let prompt: String = conversation.responses[i].0.clone();
                let response: String = conversation.responses[i].1.response.clone();

                result.push(LLMMessage {
                    role: LLMMessageType::User,
                    content: prompt,
                });
                result.push(LLMMessage {
                    role: LLMMessageType::Assistant,
                    content: response,
                });
            }
        }
        None => {
            // There is no current conversation.
            (*chats).initialise_current_conversation();
        }
    }
    // Finally the prompt
    result.push(LLMMessage {
        role: LLMMessageType::User,
        content: prompt.clone(),
    });
    chats
        .get_current_conversation_mut()
        .as_mut()
        .unwrap()
        .prompt = Some(prompt);

    result
}

/// A prompt has returned from the LLM.  Process it here
fn process_chat_response(
    chat_response: ChatResponse,
    chats: Arc<Mutex<Chats>>,
) -> Result<(), JsValue> {
    print_to_console_s(format!("process_chat_request 1: {chat_response:?}"));

    // Save this to display it
    let credit = chat_response.credit;

    let mut cas = chats.lock().unwrap();
    print_to_console("process_chat_request 1.2: ");

    // A new round to be added to the current conversation
    //cas.update_current_conversation(chat_response)?;

    // Get the cost
    let this_cost = chat_response.cost;
    let total_cost = match cas.get_current_conversation() {
        Some(c) => c.responses.iter().fold(0.0, |a, b| a + b.1.cost) + this_cost,
        None => 0.0,
    };

    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    print_to_console("process_chat_request 1.5 ");

    // Get response area and update the response
    let result_div = document.get_element_by_id("response_div").unwrap();
    print_to_console("process_chat_request 1.6 ");
    cas.update_current_conversation(chat_response)?;
    let display: String = if let Some(c) = cas.get_current_conversation() {
        let s = c.get_response_display();
        print_to_console_s(format!("process_chat_request 1.6.1 s: {s}"));
        s
    } else {
        result_div.set_inner_html("");
        print_to_console("process_chat_request 1.6.2 ");
        "".to_string()
    };
    print_to_console("process_chat_request 1.7 {display}");
    result_div.set_inner_html(display.as_str());

    result_div.set_scroll_top(result_div.scroll_height()); // Scroll to the bottom
    print_to_console("process_chat_request 1.7.5 ");
    // Store credit in chat_state so it is available for new conversations
    cas.credit = credit;
    print_to_console("process_chat_request 1.8 ");

    update_cost_display(&document, credit, total_cost, this_cost);

    print_to_console("process_chat_response 2");
    Ok(())
}

/// The callback for `make_request`
fn make_request_cb(message: Message, conversations: Arc<Mutex<Chats>>) {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    set_status(
        &document,
        format!("make_request_cb 1 {}", message.comm_type).as_str(),
    );
    match message.comm_type {
        CommType::ChatResponse => {
            print_to_console_s(format!("make_request_cb 1.1: {message:?}"));
            let chat_response: ChatResponse =
                serde_json::from_str(message.object.as_str()).unwrap();
            print_to_console("make_request_cb 1.2");
            process_chat_response(chat_response, conversations.clone()).unwrap();
        }
        CommType::InvalidRequest => {
            let inr: InvalidRequest =
                serde_json::from_str(message.object.as_str()).expect("Not an InvalidRequest");
            let document = window()
                .and_then(|win| win.document())
                .expect("Failed to get document");
            print_to_console("chat_request ivr 1");
            let result_div = document.get_element_by_id("response_div").unwrap();
            result_div.set_inner_html(&inr.reason);
        }
        _ => (),
    };
}

/// The callback for the submit button to send a prompt to the model.
fn chat_submit_cb(chats: Arc<Mutex<Chats>>) {
    print_to_console("chat_submit 1");
    // Get the contents of the prompt
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let prompt_input: HtmlInputElement = document
        .get_element_by_id("prompt_input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .map_err(|err| format!("Error casting to HtmlInputElement: {:?}", err))
        .unwrap();
    let prompt = prompt_input.value();
    prompt_input.set_value("");
    set_status(&document, format!("Sending prompt: {prompt}").as_str());

    // The history or the chat so far, plus latest prompt
    let messages: Vec<LLMMessage> = build_messages(chats.clone(), prompt.clone());

    // Get the model
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

    // Get the token
    let token: String;
    if let Some(t) = document.body().unwrap().get_attribute("data.token") {
        token = t;
    } else {
        todo!("Set status concerning error: No data token");
    }

    let chat_prompt = ChatPrompt {
        model,
        messages,
        temperature: 1.0, // Todo: Get this from user interface
        token,
    };

    let message: Message = Message::from(chat_prompt);
    print_to_console("chat_submit 2 submit: calling make_request");
    let conversation_collection = chats.clone();
    make_request(message, move |message: Message| {
        make_request_cb(message, conversation_collection.clone())
    })
    .unwrap();
}
