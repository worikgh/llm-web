//! First parameter is the mandatory port to use.
//! Certificate and private key are hardcoded to sample files.
//! hyper will automatically use HTTP/2 if a client starts talking HTTP/2,
//! otherwise HTTP/1.1 will be used.

use crate::authorisation::login;
use crate::authorisation::LoginResult;
use crate::session::Session;
use chrono::Utc;
use hyper::body;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper::{Body, Request, Response, StatusCode};
use llm_rs::json::ChatRequestInfo;
use llm_rs::openai_interface;
use llm_web_common::communication::ChatPrompt;
use llm_web_common::communication::ChatResponse;
use llm_web_common::communication::InvalidRequest;
use llm_web_common::communication::LLMMessage;
use llm_web_common::communication::LoginResponse;
use llm_web_common::communication::LogoutRequest;
use llm_web_common::communication::LogoutResponse;
use llm_web_common::communication::Message;
use llm_web_common::communication::{CommType, LoginRequest};
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use std::{env, fs, io};
use uuid::Uuid;
#[derive(Debug)]
/// Combine errors
enum ServerError {
    Serde(serde_json::Error),
    Hyper(hyper::Error),
    HyperHttp(hyper::http::Error),
}
impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> ServerError {
        ServerError::Serde(err)
    }
}

impl From<hyper::Error> for ServerError {
    fn from(err: hyper::Error) -> ServerError {
        ServerError::Hyper(err)
    }
}
impl From<hyper::http::Error> for ServerError {
    fn from(err: hyper::http::Error) -> ServerError {
        ServerError::HyperHttp(err)
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServerError::Serde(ref e) => e.fmt(f),
            ServerError::Hyper(ref e) => e.fmt(f),
            ServerError::HyperHttp(ref e) => e.fmt(f),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ServerError::Serde(ref e) => Some(e),
            ServerError::Hyper(ref e) => Some(e),
            ServerError::HyperHttp(ref e) => Some(e),
        }
    }
}

fn _error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

#[derive(Debug, Clone)]
pub struct DataServer {
    pub sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
}

impl DataServer {
    pub fn new() -> Self {
        let sessions = Arc::new(Mutex::new(HashMap::<Uuid, Session>::new()));
        Self { sessions }
    }
    pub async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // First parameter is port number (optional, defaults to 1337)
        let port = match env::args().nth(1) {
            Some(ref p) => p.to_owned(),
            None => "1337".to_owned(),
        };
        let addr = format!("127.0.0.1:{}", port).parse()?;

        let data_server = DataServer::new();
        let data_server = Arc::new(data_server);
        let service = make_service_fn(move |_: _| {
            let data_server = Arc::clone(&data_server);
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let data_server = Arc::clone(&data_server);
                    async move { Ok::<_, Infallible>(data_server.process_request(req).await.unwrap()) }
                }))
            }
        });

        let server = Server::bind(&addr).serve(service);

        server.await?;
        Ok(())

        // // Load public certificate.
        // let certs = DataServer::load_certs("certs/public.crt")?;
        // // Load private key.
        // let key = DataServer::load_private_key("certs/private4096.key")?;
        // // Build TLS configuration.

        // // Create a TCP listener via tokio.
        // let incoming = AddrIncoming::bind(&addr)?;
        // let acceptor = TlsAcceptor::builder()
        //     .with_single_cert(certs, key)
        //     .map_err(|e| error(format!("{}", e)))?
        //     .with_all_versions_alpn()
        //     .with_incoming(incoming);
        // let data_server = DataServer::new();
        // let data_server = Arc::new(data_server); // make it cloneable, replace Arc with Rc if you're not in a multithreaded context
        // let service = make_service_fn(move |_: _| {
        //     // move keyword added before |_|
        //     let data_server = Arc::clone(&data_server); // clone the server inside the closure
        //     async move {
        //         Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
        //             let data_server = Arc::clone(&data_server); // clone it again inside the next closure if this closure needs it own scope
        //             async move { Ok::<_, Infallible>(data_server.process_request(req).await.unwrap()) }
        //         }))
        //     }
        // });

        // let server = hyper::Server::builder(acceptor).serve(service);

        // // Run the future, keep going until an error occurs.
        // println!("Starting to serve on https://{}.", addr);
        // server.await?;
        // Ok(())
    }

    /// Helper function
    async fn body_to_string(
        body: Body,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Convert the body into bytes
        let bytes = body::to_bytes(body).await?;

        // Convert the bytes into a string
        let string = String::from_utf8(bytes.to_vec())
            .map_err(|err| format!("{err}: Error while converting bytes to string"))?;

        Ok(string)
    }

    /// Check that a request is valid.
    fn valid_session(&self, uuid: &Uuid, token: &str) -> bool {
        match self.sessions.lock().unwrap().get(uuid) {
            Some(session) => {
                // Found a session.  That is a start
                session.token.as_str() == token && session.expire > Utc::now()
            }
            None => false,
        }
    }

    /// All errors are transformed into Message.  TODO: Is this a good thing?
    /// Log a user out
    async fn process_logout(&self, message: &Message) -> Message {
        match message.comm_type {
            CommType::LogoutRequest => {
                let logout_request: LogoutRequest =
                    serde_json::from_str(message.object.as_str()).unwrap();
                // Get the session
                let uuid = logout_request.uuid;
                let token = logout_request.token;
                if !self.valid_session(&uuid, token.as_str()) {
                    Message::from(LogoutResponse { success: false })
                } else {
                    // A valid session
                    match self.sessions.lock().unwrap().remove(&uuid) {
                        Some(_) =>
                        // Was already logged in, but we know that
                        {
                            Message::from(LogoutResponse { success: true })
                        }
                        None => panic!("Already established uuid:{} was a valid session", uuid),
                    }
                }
            }
            _ => Message::from(InvalidRequest {
                reason: "Not a Logout Request".to_string(),
            }),
        }
    }

    /// Log a user in, or not
    async fn process_login(&self, message: &Message) -> Message {
        eprintln!("process_login(self, message: {message}) 1");
        match message.comm_type {
            CommType::LoginRequest => {
                let login_request: LoginRequest =
                    serde_json::from_str(message.object.as_str()).unwrap();
                eprintln!("process_login(self, message: {message}) 2");
                let login_result_option: Option<LoginResult> = match login(
                    login_request.username,
                    login_request.password,
                    self.sessions.clone(),
                )
                .await
                {
                    Ok(lr) => lr,
                    Err(err) => panic!("{}", err),
                };
                eprintln!("Result: {:?}", login_result_option);
                let lr = match login_result_option {
                    Some(lr) => LoginResponse {
                        success: true,
                        uuid: Some(lr.uuid),
                        token: Some(lr.token),
                    },
                    None => LoginResponse {
                        success: false,
                        uuid: None,
                        token: None,
                    },
                };
                Message {
                    comm_type: CommType::LoginResponse,
                    object: serde_json::to_string(&lr).unwrap(),
                }
            }
            _ => {
                let ir = InvalidRequest {
                    reason: format!("Can only send a LoginRequest not {}", message.comm_type),
                };
                Message {
                    comm_type: CommType::InvalidRequest,
                    object: serde_json::to_string(&ir).unwrap(),
                }
            }
        }
    }

    /// Process a chat request from the front end
    async fn process_chat_request(&self, message: &Message) -> Message {
        eprintln!("process_chat_request 1");
        if message.comm_type != CommType::ChatPrompt {
            let chat_response = InvalidRequest {
                reason: format!("Invalid message tupe sent to `chat`: {}", message.comm_type),
            };
            return Message {
                comm_type: CommType::InvalidRequest,
                object: serde_json::to_string(&chat_response).unwrap(),
            };
        }
        let response: ChatResponse = {
            eprintln!(
                "process_chat_request 1.5: {}. {:?}",
                message.comm_type, message.object
            );
            // Forced unwrap OK because comm_type is ChatPrompt
            let prompt: ChatPrompt =
                serde_json::from_str(&message.object).expect("Should be a ChatPrompt");
            let api_key = env::var("OPENAI_API_KEY").unwrap();
            // Put the conversation so far in here
            // = [Message { role, content }]
            let messages: Vec<LLMMessage> = prompt.messages;

            let data = json!({
            "messages": messages,
            "model": prompt.model.as_str(),
            "temperature": prompt.temperature,
                });

            eprintln!("process_chat_request 2");
            // Calling `openai_interface::ApiInterface::send_chat` a
            // synchronous function that blocks from an async
            // function.  This compiles.
            let chat_response: (HashMap<String, String>, ChatRequestInfo) =
                match openai_interface::ApiInterface::send_chat(api_key.as_str(), &data) {
                    Ok(response) => response,
                    Err(err) => {
                        let chat_response = InvalidRequest {
                            reason: format!("Failed `send_chat`.  Error: {err}"),
                        };
                        return Message {
                            comm_type: CommType::InvalidRequest,
                            object: serde_json::to_string(&chat_response).unwrap(),
                        };
                    }
                };
            eprintln!("process_chat_request 3");
            let mut result = "".to_string();
            result = format!("{result}Headers\n");
            for (k, v) in chat_response.0.iter() {
                result = format!("{result}{k} => {v}\n");
            }
            eprintln!("process_chat_request 4");

            let mut tokens: Vec<(String, u32)> = Vec::new();
            tokens.push((
                "prompt_tokens".to_string(),
                chat_response.1.usage.prompt_tokens,
            ));
            tokens.push((
                "completion_tokens".to_string(),
                chat_response.1.usage.completion_tokens,
            ));
            let response = chat_response.1.choices[0].message.content.clone();
            ChatResponse { tokens, response }
        };

        Message {
            comm_type: CommType::ChatResponse,
            object: serde_json::to_string(&response).unwrap(),
        }
    }

    /// Dispatch the request to subroutines
    async fn process_request(&self, req: Request<Body>) -> Result<Response<Body>, ServerError> {
        let mut response = Response::new(Body::empty());
        match (req.method(), req.uri().path()) {
            (_, "/api/login") => {
                let str = Self::body_to_string(req.into_body()).await.unwrap();
                let message: Message = match serde_json::from_str(&str) {
                    Ok(s) => s,
                    Err(err) => return Err(ServerError::from(err)),
                };

                let return_message = self.process_login(&message).await;
                let s = serde_json::to_string(&return_message).unwrap();

                *response.body_mut() = Body::from(s);
            }
            (_, "/api/chat") => {
                let str = Self::body_to_string(req.into_body()).await.unwrap();
                let message: Message = match serde_json::from_str(&str) {
                    Ok(s) => s,
                    Err(err) => return Err(ServerError::from(err)),
                };

                let return_message = self.process_chat_request(&message).await;
                let s = serde_json::to_string(&return_message).unwrap();

                *response.body_mut() = Body::from(s);
            }
            (_, "/api/logout") => {
                let str = Self::body_to_string(req.into_body()).await.unwrap();
                let message: Message = match serde_json::from_str(&str) {
                    Ok(s) => s,
                    Err(err) => return Err(ServerError::from(err)),
                };
                let return_message = self.process_logout(&message).await;
                let s = serde_json::to_string(&return_message).unwrap();
                *response.body_mut() = Body::from(s);
            }

            // Catch-all 404.
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        };
        Ok(response)
    }

    // Load public certificate from file.
    fn _load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
        // Open certificate file.
        let certfile = fs::File::open(filename)
            .map_err(|e| _error(format!("failed to open {}: {}", filename, e)))?;
        let mut reader = io::BufReader::new(certfile);

        // Load and return certificate.
        let certs = rustls_pemfile::certs(&mut reader)
            .map_err(|_| _error("failed to load certificate".into()))?;
        Ok(certs.into_iter().map(rustls::Certificate).collect())
    }

    // Load private key from file.
    fn _load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
        // Open keyfile.
        let keyfile = fs::File::open(filename)
            .map_err(|e| _error(format!("failed to open {}: {}", filename, e)))?;
        let mut reader = io::BufReader::new(keyfile);

        // Load and return a single private key.
        let keys = rustls_pemfile::rsa_private_keys(&mut reader)
            .map_err(|_| _error("failed to load private key".into()))?;
        if keys.len() != 1 {
            return Err(_error("expected a single private key".into()));
        }

        Ok(rustls::PrivateKey(keys[0].clone()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorisation;
    use crate::authorisation::delete_user;
    use crate::authorisation::tests::get_unique_user;
    use authorisation::add_user;
    use llm_web_common::communication::LoginRequest;
    use llm_web_common::communication::Message;

    fn make_request(inp: String) -> Result<Request<Body>, ServerError> {
        // Box<dyn std::error::Error>> {
        // Create a new Request with the given input as the body
        let req = Request::builder()
            .method("POST")
            .uri("http://example.com/api/login")
            .header("Content-Type", "application/json")
            .body(Body::from(inp))?;

        Ok(req)
    }
    fn make_login_request(
        username: String,
        password: String,
    ) -> Result<Request<Body>, ServerError> {
        // Only the path section of the URI is relevant
        let login_request = LoginRequest { username, password };
        let message = Message::from(login_request);
        let message = serde_json::to_string(&message).unwrap();
        make_request(message)
    }

    #[tokio::test]
    async fn login_fail() {
        // Get a user that is not in the system, and check logging in as them fails
        let username = get_unique_user("server::test::login_fail").await;
        let password = "supersecret".to_string();
        let lr = LoginRequest { username, password };
        let msg = Message {
            comm_type: CommType::LoginRequest,
            object: serde_json::to_string(&lr).unwrap(),
        };
        let server = DataServer::new();
        let result = server.process_login(&msg).await;
        eprintln!("result ({})", result,);
        assert!(result.comm_type == CommType::LoginResponse);

        let login_response: LoginResponse = serde_json::from_str(&result.object).unwrap();
        assert!(!login_response.success);
    }
    #[tokio::test]
    async fn bad_message() {
        // Check using incorrect message fails
        let username = get_unique_user("server::test::bad_message").await;
        let password = "supersecret".to_string();
        let lr = LoginRequest { username, password };
        let msg = Message {
            comm_type: CommType::ChatPrompt,
            object: serde_json::to_string(&lr).unwrap(),
        };
        let server = DataServer::new();
        let result = server.process_login(&msg).await;
        eprintln!("result.comm_type ({})", result.comm_type,);
        assert!(result.comm_type == CommType::InvalidRequest);
    }

    #[tokio::test]
    async fn server_test() {
        // Server to test
        let server = DataServer::new();

        // A user name and password to add
        let username = get_unique_user("server::test::server_test").await;
        let password = "password".to_string();
        eprintln!("Adding user: {username}/{password}");
        let b = add_user(username.as_str(), password.as_str())
            .await
            .unwrap();
        eprintln!("Assert was a successful login {b}");
        assert!(b);

        // Log them in
        let req: Request<Body> = make_login_request(username.clone(), password).unwrap();
        eprintln!("req: {:?}", req);
        let mut login_response_message = match server.process_request(req).await {
            Ok(m) => m,
            Err(err) => panic!("err: {}", err),
        };
        eprintln!("lrm: {:?}", login_response_message);

        let b = hyper::body::to_bytes(login_response_message.body_mut())
            .await
            .unwrap();
        let body_text = String::from_utf8(b.to_vec()).unwrap();
        eprintln!("body_text: {body_text}");
        let login_response_message: Message = serde_json::from_str(body_text.as_str()).unwrap();

        // Test there was the correct response
        eprintln!("Response type: {}", login_response_message.comm_type);
        assert_eq!(login_response_message.comm_type, CommType::LoginResponse);
        // Test there is at least one session
        eprintln!("One session: {}", server.sessions.lock().unwrap().len());
        assert_eq!(server.sessions.lock().unwrap().len(), 1);

        let login_response: LoginResponse =
            serde_json::from_str(login_response_message.object.as_str()).unwrap();
        // Test successful login
        eprintln!("Successful login: {}", login_response.success);
        assert!(login_response.success);

        // Log them out
        let logout_request = LogoutRequest {
            uuid: login_response.uuid.unwrap(),
            token: login_response.token.unwrap(),
        };
        let logout_request_message = Message::from(logout_request);
        let logout_response_message = server.process_logout(&logout_request_message).await;
        eprintln!(
            "Test correct message: {}",
            logout_response_message.comm_type
        );
        assert_eq!(logout_response_message.comm_type, CommType::LogoutResponse);
        // Test there is one session
        eprintln!("Zerro sessions: {}", server.sessions.lock().unwrap().len());
        assert_eq!(server.sessions.lock().unwrap().len(), 0);

        // Clean up
        delete_user(username.as_str()).await.unwrap();
    }
}
