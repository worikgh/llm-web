mod authorisation;
use authorisation::authorise;
use llm_web_common::encode_jwt_nowasm;
use llm_web_common::Claims;
use llm_web_common::LoginRequest;
use llm_web_common::LoginResponse;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
const SHARED_SECRET: &[u8] = b"i5r1 hu_#ikd 7h3 H0rf7z w98";
fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream)?;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> io::Result<usize> {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(n) => {
            let request: LoginRequest = serde_json::from_slice(&buffer[..n]).unwrap();
            let response = match process_login(request) {
                Some(claims) => {
                    // Encode `claims` into a JWT
                    let token = match encode_jwt_nowasm(&claims, SHARED_SECRET) {
                        Ok(t) => t,
                        Err(err) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("{err}:").as_str(),
                            ));
                        }
                    };
                    Some(LoginResponse { token: Some(token) })
                }
                None => None,
            };
            let response_json = serde_json::to_string(&response).unwrap();

            stream.write(response_json.as_bytes())
        }
        Err(e) => Err(e),
    }
}

/// Check that the passed user is a valid user and make a JWT whith
/// their permissions
fn process_login(request: LoginRequest) -> Option<Claims> {
    let username = request.username;
    let password = request.password;
    authorise(username, password)
}
