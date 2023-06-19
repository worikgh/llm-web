# Web Based LLM Frontend

There are three related packages, as well as `llm-rs` that handles the actual communication with the Large Language Model

1. `llm-web-fe`.  Pure rust/wam front end that runs in the browser
2. `llm-web-be`.  Proxies requests from the frontend to `llm-rs` and the replies back to the frontend.  Handles authentication and user rights also
3. `llm-web-common`.  Common code that the other two packages share

## Authentication

`llm-web-be` listens on port ??? waiting for communication from the `llm-web-fe`.
If it is sent a `LoginRequest` it returns a `LoginResponse` that holds the a JWT token as a String, or None if login refused.

See [this](https://github.com/RustCrypto/RSA) for RSA.

Signing the JWT with the `llm-web-be` private key, and verifying it with a public key is the destination....


