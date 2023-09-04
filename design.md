# Design

A `wasm` programme is loaded from the server.  It is a SPA and handles all display chores

The server responds only with data and makes no decisions about display.

Communications are initiated by the client and use XMLHttpRequests over HTTPS

##  `llm-web-fe` The Client

The wasm code that runs in the browser

Implemented from a single `index.html` that does all the wasm-bindgen heavy lifting to initialise the wasm web app in the browser

The entry point is `llm-web-fe::main()` that calls `llm-web-fe::start_app()`

## `llm-web-be` The Server

Keeps track of users and relays queries to LLMs

### Authorisation

Handled by `authorisation.rs`

## Common Code `llm-web-common`
