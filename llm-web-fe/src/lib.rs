//! Webapp front end for accessing large language models.
#[allow(unused_imports)]
use crate::utility::{print_to_console, print_to_console_s};
use set_page::initialise_page;
use set_page::set_page;
use set_page::Pages;
use wasm_bindgen::prelude::*;
mod cancel_button;
mod chat_div;
mod cost_div;
mod filters;
mod llm_webpage;
mod login_div;
mod make_request;
mod manipulate_css;
mod set_page;
mod utility;
struct MyApp {
    data: i32,
}

impl MyApp {
    fn new() -> MyApp {
        MyApp { data: 0 }
    }

    fn initialise_page(&self) -> Result<(), JsValue> {
        // Your initialization code here
        initialise_page()?;
        Ok(())
    }

    fn start_app(&mut self) -> Result<(), JsValue> {
        // Access and modify self.data as needed
        self.data += 1;

        // Call set_page() and pass self.data as needed
        set_page(Pages::LoginDiv)?;

        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    let mut app = MyApp::new();
    app.initialise_page()?;
    app.start_app()?;

    Ok(())
}
