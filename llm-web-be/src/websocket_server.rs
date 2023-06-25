use llm_web_common::communication::Message;
use serde_json::from_str;
use std::net::TcpListener;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use tungstenite::accept;
use tungstenite::protocol::WebSocket;
use websocket::stream::sync::TcpStream;
pub struct WebsocketServer {
    handle: JoinHandle<()>,
}

impl WebsocketServer {
    pub fn new(port: usize, rx: Receiver<Message>, ty: Sender<Message>) -> Self {
        // Start the server that runs for the life of the programme
        let handle = thread::spawn(move || {
            let address = format!("0.0.0.0:{port}");
            let tcp_listener: TcpListener = TcpListener::bind(address).unwrap();

            for stream in tcp_listener.incoming() {
                // A new connection from a front end
                match stream {
                    Ok(stream) => {
                        // Get the websocket
                        let ws: WebSocket<TcpStream> = match accept(stream) {
                            Ok(ws) => ws,
                            Err(err) => {
                                eprintln!("{err}: Failed to open `WebSocket`");
                                continue;
                            }
                        };
                        let ws_arc = Arc::new(Mutex::new(ws));

                        // Clone the sender for each connection
                        let ty = ty.clone();
                        // Shared websocket
                        let ws_arc_c = ws_arc.clone();

                        // Spawn a new thread to handle the WebSocket
                        // connection, sending incomming WebSocket
                        // Messages back to the main thread
                        thread::spawn(move || loop {
                            let s: tungstenite::Message = match ws_arc_c
                                .lock()
                                .unwrap()
                                .read_message()
                            {
                                Ok(m) => m,
                                Err(err) => {
                                    eprint!("{err}: Failed to read a `tungstenite::Message` from `WebSocket`" );
                                    continue;
                                }
                            };

                            let msg_s: String = match s {
                                tungstenite::Message::Text(s) => s,
                                _ => {
                                    eprintln!("Message Not Understood: {s}");
                                    continue;
                                }
                            };
                            let message: Message = match from_str(msg_s.as_str()) {
                                Ok(m) => m,
                                Err(err) => {
                                    eprintln!("{err}: Could not create `Message` from: {msg_s}");
                                    continue;
                                }
                            };
                            // Send message back to the main thread.
                            match ty.send(message) {
                                Ok(()) => (),
                                Err(err) => {
                                    eprint!("{err}: Failed to send message back to main thread");
                                    continue;
                                }
                            }
                        });

                        let ws_arc_c = ws_arc.clone();
                        // Spawn a thread to send messages from the
                        // main thread to the WebSocket
                        let _ws_send_thread = thread::spawn(move || loop {
                            // A `Message` to send.
                            match rx.recv() {
                                Ok(msg) => {
                                    // Convert to a String for sending on WebSocket
                                    let json: String = match serde_json::to_string(&msg) {
                                        Ok(s) => s,
                                        Err(err) => {
                                            eprintln!("{err}: Could not convert Message to String");
                                            continue;
                                        }
                                    };
                                    match ws_arc_c
                                        .lock()
                                        .unwrap()
                                        .write_message(tungstenite::Message::Text(json))
                                    {
                                        Ok(()) => (),
                                        Err(err) => {
                                            eprintln!("{err}: Error writing message");
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => eprintln!("{e}: Error receiving data from main thread"),
                            };
                        });
                    }
                    Err(err) => eprintln!("{err}: Failed connection"),
                };
            }
        });
        Self { handle }
    }
}
