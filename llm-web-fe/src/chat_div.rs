use crate::filters::text_for_html;
use crate::llm_webpage::LlmWebPage;
use crate::make_request::make_request;
use crate::manipulate_css::add_css_rule;
use crate::manipulate_css::clear_css;
use crate::manipulate_css::get_css_rules;
use crate::manipulate_css::set_css_rules;
use crate::set_page::new_button;
use crate::set_page::set_focus_on_element;
use crate::set_page::set_status;
use crate::set_page::update_cost_display;
#[allow(unused_imports)]
use crate::utility::{print_to_console, print_to_console_s};
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
//use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::KeyboardEvent;
use web_sys::{Event, XmlHttpRequest};

use wasm_bindgen::prelude::*;
use web_sys::{
    window, Document, Element, HtmlButtonElement, HtmlImageElement, HtmlInputElement,
    HtmlOptionElement, HtmlSelectElement,
};

/// A conversation.  If `prompt` is not `None` a chat prompt has been
/// sent and a reply is being waited for
#[derive(Debug, Deserialize, Serialize)]
struct Conversation {
    cost: f64,
    key: usize,
    prompt: Option<String>,
    responses: Vec<(String, ChatResponse)>,
    #[serde(skip_serializing, skip_deserializing)]
    request: Option<XmlHttpRequest>,
}

impl Conversation {
    fn new(key: usize) -> Self {
        Self {
            cost: 0.0,
            key,
            prompt: None,
            responses: Vec::new(),
            request: None,
        }
    }

    /// Get a display to put in response area.  Transform the text
    /// into HTML, and put class definitions in for prompts and
    /// responses so they can be styled
    fn get_response_display(&self) -> String {
        let mut result = String::new();
        for i in self.responses.iter() {
            let prompt = i.0.as_str();
            let prompt = text_for_html(prompt);
            let respone = i.1.response.as_str();
            let respone = text_for_html(respone);
            result = format!("{result}<br/><span class='prompt'>{} {prompt}</span><br/><span class='response'>{respone}</span>",i.1.model);
        }
        result
    }

    /// Get the label to put on this conversation.  The text to
    /// display is taken from the first prompt for the conversation.
    /// It is hard to know what to do here.  Perhaps a method for the
    /// user to name conversations?
    fn get_label(&self) -> String {
        if self.responses.is_empty() {
            format!("{}: Empty conversation", self.key)
        } else {
            format!(
                "{}: {}",
                self.key,
                self.responses.first().unwrap().0.clone()
            )
        }
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

    fn new_conversation_key(&self) -> usize {
        // Generate a index for the conversation.  This will ensure
        // there are usize::MAX conversations, ever, during the life
        // time of this interface
        match self.conversations.keys().len() {
            0 => {
                // Base case.  First name
                0
            }
            _ => {
                let indexes = self.conversations.keys().collect::<Vec<&usize>>();
                // TODO There has to be a better way to get the maximum index already
                let max = indexes.iter().fold(0, |a, b| if **b > a { **b } else { a });
                max + 1
            }
        }
    }

    fn delete_conversation(&mut self, key: usize) {
        if self.conversations.remove(&key).is_none() {
            print_to_console_s(format!(
                "Chats::delete conversation: Key {key} does not exist"
            ));
        }
        if let Some(cc) = self.current_conversation {
            if cc == key {
                self.current_conversation = None;
            }
        }
    }

    // The current conversation is where the focus of the user is. It
    // must be:
    // * initialised when a conversation starts .
    // * updated when a response received
    // * References to read it
    // * Reference to mutate it
    fn initialise_current_conversation(&mut self) {
        let index = self.new_conversation_key();
        self.conversations.insert(index, Conversation::new(index));
        self.current_conversation = Some(index);
    }

    /// Triggered by the radio buttons
    fn set_current_conversation(&mut self, cc: usize) -> Result<(), JsValue> {
        if self.conversations.get(&cc).is_some() {
            self.current_conversation = Some(cc);
            Ok(())
        } else {
            Err(JsValue::from_str("invalid conversation"))
        }
    }

    /// Update conversation when response received
    /// Possibly never used
    fn _update_current_conversation(&mut self, response: ChatResponse) -> Result<(), JsValue> {
        // Preconditions:
        // 1. There is a current conversation
        // 2. The `prompt` is not None in current conversation
        let conversation = self.get_current_conversation_mut().unwrap();
        let prompt: String = conversation.prompt.as_ref().unwrap().clone();
        conversation.prompt = None;
        conversation.responses.push((prompt, response));
        Ok(())
    }

    /// Update a conversation when response received
    fn update_conversation(
        &mut self,
        response: ChatResponse,
        conversation_key: usize,
        conversation_cost: f64,
    ) -> Result<(), JsValue> {
        // Preconditions:
        // 1. There is a current conversation
        // 2. The `prompt` is not None in current conversation
        let conversation = self.get_conversation_mut(conversation_key).unwrap();
        conversation.cost = conversation_cost;
        let prompt: String = conversation.prompt.as_ref().unwrap().clone();
        conversation.prompt = None;
        conversation.responses.push((prompt, response));
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

    /// Get a conversation to mutate
    fn get_conversation_mut(&mut self, current_conversation: usize) -> Option<&mut Conversation> {
        self.conversations.get_mut(&current_conversation)
    }

    /// Get a conversation to read
    fn get_conversation(&self, current_conversation: usize) -> Option<&Conversation> {
        self.conversations.get(&current_conversation)
    }

    /// Check that a conversation exists
    fn conversation_exists(&self, key: usize) -> bool {
        self.conversations.get(&key).is_some()
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
        let chats = Rc::new(RefCell::new(Chats::new()?));

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
        prompt_inp.set_attribute("spellcheck", "true")?;
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
        let closure_onclick =
            Closure::wrap(Box::new(move || chat_submit_cb(cc.clone())) as Box<dyn Fn()>);
        submit_button.set_onclick(Some(closure_onclick.as_ref().unchecked_ref()));
        closure_onclick.forget();

        prompt_div.append_child(&submit_button)?;

        let side_panel_div = make_side_panel(document, chats.clone())?;

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
        add_css_rule(document, "#prompt_div", "border", "1px solid black")?;
        add_css_rule(document, "#prompt_div", "display", "flex")?;
        add_css_rule(document, "#prompt_div", "align-items", "center")?;
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
        add_css_rule(document, "#side-panel-div", "border", "1px solid black")?;

        // Pad the button to the left
        add_css_rule(document, "#chat-submit", "margin-left", "1em")?;

        set_focus_on_element(document, "prompt_input");

        // cancel_button.set_id(format!("cancel_request_{key}").as_str());
        // cancel_button.set_attribute("class", "prompt_cancel_button")?;
        // padding: 10px;
        // add_css_rule(document, ".prompt_cancel_button", "padding", "10%")?;
        // border: none;
        // add_css_rule(document, ".prompt_cancel_button", "border", "none")?;
        // background-color: #f0f0f0;
        add_css_rule(
            // Transparent
            document,
            ".prompt_cancel_button",
            "background-color",
            "rgba(0, 0, 0, 0)",
        )?;
        // color: #333;
        // add_css_rule(document, ".prompt_cancel_button", "color", "#333")?;
        // font-size: 16px;
        // add_css_rule(document, ".prompt_cancel_button", "font-size", "16px")?;
        // cursor: pointer;
        // add_css_rule(document, ".svg_cancel_button", "cursor", "pointer")?;
        // add_css_rule(document, ".svg_cancel_button", "width", "20px")?;
        add_css_rule(document, ".prompt_cancel_button", "height", "1.5em")?;

        // add_css_rule(document, ".prompt_cancel_button", "display", "flex")?;
        // add_css_rule(
        //     document,
        //     ".prompt_cancel_button",
        //     "justify-content",
        //     "center",
        // )?;

        // Align the cancel button vertically
        add_css_rule(document, "li", "display", "flex")?;
        add_css_rule(document, "li", "align-items", "center")?;
        add_css_rule(document, ".delete_conversation_button", "height", "1.0em")?;

        add_css_rule(document, ".cost_span", "font-size", "small")?;
        add_css_rule(document, ".conversation_name", "font-size", "small")?;
        add_css_rule(document, ".conversation_name", "width", "65%")?;
        add_css_rule(document, ".conversation_name", "display", "inline-block")?;
        add_css_rule(document, ".conversation_name", "overflow", "hidden")?;
        add_css_rule(document, ".conversation_name", "white-space", "nowrap")?;
        add_css_rule(document, ".conversation_name", "margin-right", ".4em")?;
        add_css_rule(document, "ul", "list-style", "none")?;

        Ok(chat_div)
    }
}

/// Make a new conversation
fn make_new_conversation(chats: Rc<RefCell<Chats>>) -> Result<usize, JsValue> {
    match chats.try_borrow_mut() {
        Err(err) => {
            let result = format!("Failed to borrow chats making a new conversation: {err}");
            return Err(JsValue::from_str(result.as_str()));
        }
        Ok(mut chats) => {
            let key = chats.new_conversation_key();
            chats.conversations.insert(key, Conversation::new(key));
            Ok(key)
        }
    }
}

/// Update the current conversation
fn set_current_conversation(chats: Rc<RefCell<Chats>>, key: usize) {
    if let Ok(mut chats) = chats.try_borrow_mut() {
        match chats.set_current_conversation(key) {
            Ok(()) => (),
            Err(err) => print_to_console_s(format!(
                "Failed to set current conversation to: {key}. Er: {err:?}"
            )),
        };
    } else {
        print_to_console("Failed to borrow chats mut set_currrent_conversation");
    }
}

/// A prompt has returned from the LLM.  Process it here
fn process_chat_response(
    chat_response: ChatResponse,
    chats: Rc<RefCell<Chats>>,
    conversation_key: usize,
) -> Result<(), JsValue> {
    print_to_console_s(format!("process_chat_request 1: {chat_response:?}"));

    // Check if conversation has been deleted while the LLM was working
    if !match chats.try_borrow() {
        Ok(chats_ref) => chats_ref.conversation_exists(conversation_key),
        Err(err) => {
            print_to_console_s(format!(
                "Failed to borrow chats `process_chat_response`: {err:?}"
            ));
            false
        }
    } {
        return Err(JsValue::from_str(
            "Conversation deleted while waiting for response?",
        ));
    }

    // Save this to display it
    let credit = chat_response.credit;

    // A new round to be added to the current conversation
    //cas.update_current_conversation(chat_response)?;

    // Get the cost
    let this_cost = chat_response.cost;
    let conversation_cost = match chats.try_borrow() {
        Err(err) => {
            print_to_console_s(format!(
                "Failed to borrow chats `process_chat_response`: {err:?}"
            ));
            f64::NAN
        }
        Ok(cas) => match cas.get_conversation(conversation_key) {
            Some(c) => c.responses.iter().fold(0.0, |a, b| a + b.1.cost) + this_cost,
            None => 0.0,
        },
    };

    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");

    match chats.try_borrow_mut() {
        Err(err) => print_to_console_s(format!(
            "Failed to borrow chats `process_chat_response`: {err:?}"
        )),
        Ok(mut cas) => {
            cas.credit = credit;
            cas.update_conversation(chat_response, conversation_key, conversation_cost)?;
            if let Some(cc) = cas.current_conversation {
                // There is a current conversation
                if cc == conversation_key {
                    // This data returned is for the current
                    // conversation So update display
                    // Get response area and update the response
                    let result_div = document.get_element_by_id("response_div").unwrap();
                    let display: String = if let Some(c) = cas.get_conversation(conversation_key) {
                        c.get_response_display()
                    } else {
                        print_to_console_s(format!(
                            "Cannot get the current conversation: {conversation_key}"
                        ));
                        result_div.set_inner_html("");
                        "".to_string()
                    };
                    result_div.set_inner_html(display.as_str());

                    // Scroll to the bottom
                    result_div.set_scroll_top(result_div.scroll_height());
                }
            }
        }
    };

    update_cost_display(&document, credit, this_cost);

    Ok(())
}

fn clear_response_screen() {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let result_div = document.get_element_by_id("response_div").unwrap();
    result_div.set_inner_html("");
}
/// Display the current conversation or clear the response screen if
/// there is none
fn update_response_screen(conversation: &Conversation) {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let result_div = document.get_element_by_id("response_div").unwrap();
    let display: String = conversation.get_response_display();
    result_div.set_inner_html(display.as_str());
    // Scroll to the bottom
    result_div.set_scroll_top(result_div.scroll_height());
}

/// The callback for abort fetching a response
fn abort_request_cb() {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    set_status(&document, "Abort request");
}

/// The callback for `make_request`
fn make_request_cb(
    message: Message,
    conversations: Rc<RefCell<Chats>>,
    current_conversation: usize,
) {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    set_status(
        &document,
        format!("make_request_cb 1 {}", message.comm_type).as_str(),
    );
    match message.comm_type {
        CommType::ChatResponse => {
            let chat_response: ChatResponse =
                serde_json::from_str(message.object.as_str()).unwrap();
            process_chat_response(chat_response, conversations.clone(), current_conversation)
                .unwrap();
            remake_side_panel(conversations.clone()).unwrap();
        }
        CommType::InvalidRequest => {
            let inr: InvalidRequest =
                serde_json::from_str(message.object.as_str()).expect("Not an InvalidRequest");
            let document = window()
                .and_then(|win| win.document())
                .expect("Failed to get document");

            let result_div = document.get_element_by_id("response_div").unwrap();
            result_div.set_inner_html(&inr.reason);
        }
        _ => (),
    };
}

/// The callback for the submit button to send a prompt to the model.
fn chat_submit_cb(chats: Rc<RefCell<Chats>>) {
    // print_to_console("chat_submit 1");
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
    // The model to use
    let model = get_model();

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

    // Need to tell the callback for `make_request` what conversation
    // is being used.  Cannot rely "current_convesation" as it may
    // change while the network request is under way
    let current_conversation: usize = match chats.try_borrow() {
        Err(_err) => {
            print_to_console(
                "Failed borrowing chats to get current conversation for request callback",
            );
            return;
        }
        Ok(chats) => match chats.current_conversation {
            Some(cc) => cc,
            None => {
                print_to_console(
                    "Failed borrowing chats to get current conversation for request callback",
                );
                return;
            }
        },
    };
    let chats_make_req_cb = chats.clone();
    let xhr = make_request(
        message,
        move |message: Message| {
            make_request_cb(message, chats_make_req_cb.clone(), current_conversation)
        },
        abort_request_cb,
    )
    .unwrap();
    match chats.try_borrow_mut() {
        Err(err) => print_to_console_s(format!("Failed to borrow chats `chat_submit_cb`: {err:?}")),
        Ok(mut chats) => chats.get_current_conversation_mut().unwrap().request = Some(xhr),
    };

    remake_side_panel(chats.clone()).unwrap();
}

/// Callback for conversation select
fn select_conversation_cb(event: Event, chats_clone: Rc<RefCell<Chats>>) {
    let target = event.target().unwrap();
    let target_element = target.dyn_ref::<web_sys::HtmlElement>().unwrap();

    // Get the ID off the clicked radio button
    let id = target_element.id();

    // Radio: conversation_radio_1
    let id = id.as_str();
    let id = &id["conversation_radio_".len()..];
    match id.parse() {
        Ok(key) => {
            // clear_current_conversation();
            match chats_clone.try_borrow_mut() {
                Ok(mut chats) => match chats.set_current_conversation(key) {
                    Ok(()) => (),
                    Err(_err) => {
                        print_to_console_s(format!("Failed to set current conversation to: {key}"))
                    }
                },
                Err(_err) => {
                    print_to_console("Cannot borrow_mut chats current_radio_click handler")
                }
            };
            // Redraw the response screen
            match chats_clone.try_borrow() {
                Ok(chats_ref) => update_response_screen(chats_ref.conversations.get(&key).unwrap()),
                Err(err) => print_to_console_s(format!(
                    "select_conversation_cb: Failed to clone chats: {err:?}"
                )),
            }
        }
        Err(err) => print_to_console_s(format!("Cannot parse id: {id}. Error: {err:?}")),
    };

    //....
}

/// New conversation callback
fn new_conversation_callback(chats_clone: Rc<RefCell<Chats>>) {
    match make_new_conversation(chats_clone.clone()) {
        Ok(key) => {
            set_current_conversation(chats_clone.clone(), key);
            // Make the new conversation the current.  FIXME:  This is convoluted
            match chats_clone.try_borrow() {
                Ok(chats_ref) => update_response_screen(chats_ref.conversations.get(&key).unwrap()),
                Err(err) => print_to_console_s(format!(
                    "new_conversation_callback: Failed to clone chats: {err:?}"
                )),
            }
        }
        Err(err) => print_to_console_s(format!("Failed to make new conversation: {err:?}")),
    }
    match remake_side_panel(chats_clone.clone()) {
        Ok(()) => (),
        Err(err) => print_to_console_s(format!("Failed to remake side panel: {err:?}")),
    };
}

/// Style experiment button
fn style_experiment_cb() {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    let mut cs_rules = get_css_rules(&document).unwrap();
    cs_rules
        .insert("#side-panel-div", "background-color", "aliceblue")
        .unwrap();
    match clear_css(&document) {
        Ok(()) => (),
        Err(err) => print_to_console_s(format!(
            "Failed clear_css {}:{}",
            err.as_string().unwrap_or("<UNKNOWN>".to_string()),
            err.js_typeof().as_string().unwrap_or("".to_string()),
        )),
    };
    set_css_rules(&document, &cs_rules).unwrap();
}

/// Calcel button callback
fn cancel_cb(chats_clone: Rc<RefCell<Chats>>) {
    match chats_clone.try_borrow_mut() {
        Ok(mut m_chats) => {
            match m_chats.get_current_conversation() {
                Some(cc) => match &cc.request {
                    Some(xhr) => {
                        xhr.abort().unwrap();
                        if let Some(cc) = m_chats.get_current_conversation_mut() {
                            cc.prompt = None;
                            cc.request = None;
                        } else {
                            print_to_console("Cannot get current conversation for abort");
                        }
                    }
                    None => print_to_console("Got no xhr"),
                },
                None => print_to_console("Got no cc"),
            };
        }
        Err(err) => print_to_console_s(format!(
            "Failed to borrow chats `make_conversation_list` {err:?}"
        )),
    };
}

/// Callback for conversation delete button
fn delete_conversation_cb(event: Event, chats_clone: Rc<RefCell<Chats>>) {
    let target = event.target().unwrap();
    let target_element = target.dyn_ref::<web_sys::HtmlElement>().unwrap();

    // Get the ID off the clicked radio button
    let id = target_element.id();
    let id = id.as_str();
    let id = &id["delete_conversation_".len()..];
    match id.parse::<usize>() {
        Err(err) => print_to_console_s(format!(
            "Cannot parse {id} setting up delete conversation button: Error: {err}"
        )),
        Ok(key) => {
            // `key` is the conversation to delete
            match chats_clone.try_borrow_mut() {
                Err(_err) => {
                    print_to_console("Delete conversation handler faied to borrow mut chats")
                }
                Ok(mut chats_mut) => {
                    if let Some(cc) = chats_mut.current_conversation {
                        if cc == key {
                            // Deleting current conversation
                            clear_response_screen();
                        }
                    }
                    chats_mut.delete_conversation(key);
                }
            };

            if let Err(_err) = remake_side_panel(chats_clone.clone()) {
                print_to_console("Delete conversation handler remake the side panel");
            }
        }
    };
}

/// Called to construct the messages for a request.  Each interaction
/// with the LLM includes a history of prevous interactions.  In the
/// general case this is the history of the current conversation.
/// `prompt` is the user's latest input
fn build_messages(chats: Rc<RefCell<Chats>>, prompt: String) -> Vec<LLMMessage> {
    // `messages` is the historical response, build it here.
    let mut result: Vec<LLMMessage> = Vec::new();

    // The "role" is first.  Allways using the same role (TODO: this
    // needs to be configurable)
    result.push(LLMMessage {
        role: LLMMessageType::System,
        content: "You are a helpful assistant".to_string(),
    });

    match chats.try_borrow_mut() {
        Err(err) => print_to_console_s(format!("Failed to borrow chats `build_messages` {err:?}")),
        Ok(mut chats) => {
            // Then the history of the conversation
            match chats.get_current_conversation() {
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
        }
    };
    result
}

/// Remake the side panel
fn remake_side_panel(chats: Rc<RefCell<Chats>>) -> Result<(), JsValue> {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");

    // Get the data from the side-panel that have changed from defaults
    // Model
    let model = get_model();

    let new_side_panel_div = make_side_panel(&document, chats.clone())?;
    let old_side_panel = document
        .get_element_by_id("side-panel-div")
        .ok_or_else(|| JsValue::from_str("Failed to get side panel."))?;
    let parent = old_side_panel
        .parent_node()
        .ok_or_else(|| JsValue::from_str("Failed to find parent node."))?;
    parent.replace_child(&new_side_panel_div, &old_side_panel)?;

    // Reset the data that may have changed from the defaults
    set_model(model.as_str());
    Ok(())
}

/// Create the side panel
fn make_side_panel(document: &Document, chats: Rc<RefCell<Chats>>) -> Result<Element, JsValue> {
    // The side_panel menu
    // print_to_console("make_side_panel 1");
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
    select_element.set_id("model_chat");
    let options = select_element.options();

    options.add_with_html_option_element(&HtmlOptionElement::new_with_text_and_value(
        "Gpt-3",
        "gpt-3.5-turbo",
    )?)?;

    options.add_with_html_option_element(&HtmlOptionElement::new_with_text_and_value(
        "Gpt-4", "gpt-4",
    )?)?;
    side_panel_div.append_child(&select_element)?;

    // New conversation button
    let new_conversation = new_button(document, "new_conversation", "New Conversation")?;
    let chats_clone = chats.clone();
    let new_conversation_closure = Closure::wrap(Box::new(move || {
        new_conversation_callback(chats_clone.clone());
    }) as Box<dyn Fn()>);

    new_conversation.set_onclick(Some(new_conversation_closure.as_ref().unchecked_ref()));
    new_conversation_closure.forget();

    side_panel_div.append_child(&new_conversation)?;
    // Experimental button
    let clear_style = new_button(document, "clear_style", "Style Experiment")?;
    let resp_closure = Closure::wrap(Box::new(|| {
        style_experiment_cb();
    }) as Box<dyn Fn()>);

    clear_style.set_onclick(Some(resp_closure.as_ref().unchecked_ref()));
    resp_closure.forget();
    side_panel_div.append_child(&clear_style)?;

    let conversation_list = make_conversation_list(document, chats.clone())?;
    side_panel_div.append_child(&conversation_list)?;

    Ok(side_panel_div)
}

/// Make a list of conversations for the side panel
fn make_conversation_list(
    document: &Document,
    chats: Rc<RefCell<Chats>>,
) -> Result<Element, JsValue> {
    // print_to_console("make_conversation_list 1");

    let conversation_list_div = document.create_element("div")?;

    // Collect the data to build the display widgets (<li>...</li>)
    // for the conversations
    struct DisplayData {
        active: bool,
        current: bool,
        label: String,
        cost: f64,
    }

    let mut conversation_displays: HashMap<usize, DisplayData> = HashMap::new();
    match chats.clone().try_borrow() {
        Err(_err) => {
            return Err(JsValue::from_str(
                "Cannot borrow chats.  make_conversation_list",
            ))
        }
        Ok(chats) => {
            let conversations = &chats.conversations;

            for (key, conversation) in conversations.iter() {
                // Is this converstion active?
                // Is this the currentconversation?

                let active = conversation.prompt.is_some();
                let current = match chats.current_conversation {
                    Some(c) => c == *key,
                    None => false,
                };
                let label = conversation.get_label();
                let cost = conversation.cost;
                conversation_displays.insert(
                    *key,
                    DisplayData {
                        cost,
                        active,
                        current,
                        label,
                    },
                );
            }
        }
    }

    // Now buld the HTML data
    let ul = document.create_element("ul")?;

    let mut keys: Vec<&usize> = conversation_displays.keys().collect();
    keys.sort();
    for key in keys {
        let dd = conversation_displays.get(key).unwrap();

        //...........conversation_displays
        // Each conversation has an element in this list
        let li = document.create_element("li")?;

        // Display the cost
        let cost = format!("{:0>.1}\u{00A2}", dd.cost);
        let cost_span = document.create_element("span")?;
        cost_span.set_inner_html(cost.as_str());
        cost_span.set_attribute("class", "cost_span")?;
        li.append_child(&cost_span)?;

        let delete_button: HtmlImageElement = document
            .create_element("img")
            .map_err(|err| format!("Error creating button element: {:?}", err))?
            .dyn_into::<HtmlImageElement>()
            .map_err(|err| format!("Error casting to HtmlButtonElement: {:?}", err))?;

        delete_button.set_src("data/trash.png");
        delete_button.set_id(format!("delete_conversation_{key}").as_str());
        delete_button.set_attribute("class", "delete_conversation_button")?;

        // Set event handler
        let chats_clone = chats.clone();
        let event_handler = Closure::wrap(Box::new(move |event: Event| {
            delete_conversation_cb(event, chats_clone.clone());
        }) as Box<dyn FnMut(_)>);

        delete_button.set_onclick(Some(event_handler.as_ref().unchecked_ref()));
        event_handler.forget();

        li.append_child(&delete_button)?;

        // A radio button The current conversation is selected.
        // Changing the selection will change the current
        // conversation.
        let current_radio = document.create_element("input")?;
        let current_radio = current_radio.dyn_ref::<HtmlInputElement>().unwrap();
        current_radio.set_attribute("type", "radio")?;
        current_radio.set_attribute("name", "conversation_radio_buttons")?;
        current_radio.set_id(format!("conversation_radio_{key}").as_str());
        if dd.current {
            current_radio.set_attribute("checked", "")?;
        }
        let chats_clone = chats.clone();
        let current_radio_click = Closure::wrap(Box::new(move |event: web_sys::Event| {
            select_conversation_cb(event, chats_clone.clone());
        }) as Box<dyn FnMut(Event)>);

        current_radio.set_onclick(Some(current_radio_click.as_ref().unchecked_ref()));
        current_radio_click.forget();
        li.append_child(current_radio)?;

        // The text to display is taken from the first prompt for the
        // conversation.  It is hard to know what to do here.  Perhaps
        // a method for the user to name conversations?
        let conversation_name = document.create_element("span")?;
        conversation_name.set_attribute("class", "conversation_name")?;
        conversation_name.set_inner_html(dd.label.as_str());
        li.append_child(&conversation_name)?;

        // If this is active create a button to cancel it
        if dd.active {
            // Cancel button
            let cancel_button: HtmlImageElement = document
                .create_element("img")
                .map_err(|err| format!("Error creating button element: {:?}", err))?
                .dyn_into::<HtmlImageElement>()
                .map_err(|err| format!("Error casting to HtmlButtonElement: {:?}", err))?;
            // cancel_button.set_inner_text("cancel");
            cancel_button.set_src("data/cancel_button.png");
            cancel_button.set_id(format!("cancel_request_{key}").as_str());
            cancel_button.set_attribute("class", "prompt_cancel_button")?;
            li.append_child(&cancel_button)?;

            let chats_clone = chats.clone();
            let event_handler = Closure::wrap(Box::new(move |_event: Event| {
                cancel_cb(chats_clone.clone());
                remake_side_panel(chats_clone.clone()).unwrap();
            }) as Box<dyn FnMut(_)>);

            cancel_button.set_onclick(Some(event_handler.as_ref().unchecked_ref()));
            event_handler.forget();
        }
        ul.append_child(&li)?;
    }
    conversation_list_div.append_child(&ul)?;
    Ok(conversation_list_div)
}

/// Get the model that the user has selected from the side panel
fn get_model() -> String {
    // Worik: I am having a debate with myself: Should the `document`
    // be passed around or should it be grabbed from the global
    // environment each time?

    // The former would be strictly necessary if `window()` can ever
    // be one thing or another - but I do not think that is the case.

    // The later makes for simpler function signatures (less typing)
    // and if `window()` always produces the same object everywhere
    // does not harm except...

    // Is obtaining the `window()` expensive?  How do I profile
    // something like that?
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    // Get the model
    let model_selection: HtmlSelectElement = document
        .get_element_by_id("model_chat")
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .map_err(|err| format!("Error casting to HtmlOptionsCollection: {err:?}",))
        .unwrap();
    let model: String = if let Some(element) = model_selection.selected_options().item(0) {
        element.get_attribute("value").unwrap()
    } else {
        // This should never happen.  There is a default and no way to
        // select no model.
        print_to_console("Cannot get a model from the side panel");
        panic!()
    };
    model
}

/// Set the model to use.
fn set_model(new_model: &str) {
    let document = window()
        .and_then(|win| win.document())
        .expect("Failed to get document");
    // Get the model
    let model_selection: HtmlSelectElement = document
        .get_element_by_id("model_chat")
        .unwrap()
        .dyn_into::<HtmlSelectElement>()
        .map_err(|err| format!("Error casting to HtmlOptionsCollection: {err:?}",))
        .unwrap();
    for i in 0..model_selection.length() {
        // Forced unwrap is guarded by loop
        let e = model_selection.get(i).unwrap();
        if e.get_attribute("value").unwrap() == new_model {
            model_selection.set_selected_index(i as i32);
            return;
        }
    }
    // Get to here and there has been an error.
    print_to_console_s(format!("set_model({new_model}) failed"));
}
