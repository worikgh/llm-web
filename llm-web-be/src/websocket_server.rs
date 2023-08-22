//! A chat server that broadcasts a message to all connections.
//!
//! This is a simple line-based server which accepts WebSocket connections,
//! reads lines from those connections, and broadcasts the lines to all other
//! connected clients.
//!
//! You can test this out by running:
//!
//!     cargo run --example server 127.0.0.1:12345
//!
//! And then in another window run:
//!
//!     cargo run --example client ws://127.0.0.1:12345/
//!
//! You can run the second command in multiple windows and then chat between the
//! two, seeing the messages from the other client as they're received. For all
//! connected clients they'll all join the same room and see everyone else's
//! messages.

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::sync::mpsc::Sender;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tungstenite::protocol::WebSocket;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
pub struct WebsocketServer;
impl WebsocketServer {
    pub fn new(_: usize, _: Sender<WebSocket<TcpStream>>) -> Self {
        Self
    }
}
async fn _handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let peers = peer_map.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(_handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}
// //use llm_web_common::communication::Message;
// //use serde_json::from_str;
// use std::net::TcpListener;
// use std::sync::mpsc::Sender;
// //use std::sync::{Arc, Mutex};
// use std::net::TcpStream;
// use std::thread;
// //use std::thread::JoinHandle;
// use tungstenite::accept;
// use tungstenite::protocol::WebSocket;
// pub struct WebsocketServer {
//     // _handle: JoinHandle<()>,
// }

// impl WebsocketServer {
//     pub fn new(port: usize, tw: Sender<WebSocket<TcpStream>>) -> Self {
//         // Start the server that runs for the life of the programme
//         let _handle = thread::spawn(move || {
//             let address = format!("0.0.0.0:{port}");
//             let tcp_listener: TcpListener = TcpListener::bind(address).unwrap();

//             for stream in tcp_listener.incoming() {
//                 // A new connection from a front end
//                 match stream {
//                     Ok(stream) => {
//                         // Get the websocket
//                         let ws = match accept(stream) {
//                             Ok(ws) => ws,
//                             Err(err) => {
//                                 eprintln!("{err}: Failed to open `WebSocket`");
//                                 continue;
//                             }
//                         };
//                         if let Err(err) = tw.send(ws) {
//                             // Failed to send the socket
//                             eprintln!("{err}: Failed to send websocket to server");
//                         }
//                         // let ws_arc = Arc::new(Mutex::new(ws));

//                         // // Clone the sender for each connection
//                         // let ty = ty.clone();
//                         // // Shared websocket
//                         // let ws_arc_c = ws_arc.clone();

//                         // // Spawn a new thread to handle the WebSocket
//                         // // connection, sending incomming WebSocket
//                         // // Messages back to the main thread
//                         // thread::spawn(move || loop {
//                         //     let s: tungstenite::Message = match ws_arc_c
//                         //         .lock()
//                         //         .unwrap()
//                         //         .read_message()
//                         //     {
//                         //         Ok(m) => m,
//                         //         Err(err) => {
//                         //             eprint!("{err}: Failed to read a `tungstenite::Message` from `WebSocket`" );
//                         //             continue;
//                         //         }
//                         //     };

//                         //     let msg_s: String = match s {
//                         //         tungstenite::Message::Text(s) => s,
//                         //         _ => {
//                         //             eprintln!("Message Not Understood: {s}");
//                         //             continue;
//                         //         }
//                         //     };
//                         //     let message: Message = match from_str(msg_s.as_str()) {
//                         //         Ok(m) => m,
//                         //         Err(err) => {
//                         //             eprintln!("{err}: Could not create `Message` from: {msg_s}");
//                         //             continue;
//                         //         }
//                         //     };
//                         //     // Send message back to the main thread.
//                         //     match ty.send(message) {
//                         //         Ok(()) => (),
//                         //         Err(err) => {
//                         //             eprint!("{err}: Failed to send message back to main thread");
//                         //             continue;
//                         //         }
//                         //     }
//                         // });

//                         // let ws_arc_c = ws_arc.clone();
//                         // // Spawn a thread to send messages from the
//                         // // main thread to the WebSocket
//                         // let _ws_send_thread = thread::spawn(move || loop {
//                         //     // A `Message` to send.
//                         //     match rx.recv() {
//                         //         Ok(msg) => {
//                         //             // Convert to a String for sending on WebSocket
//                         //             let json: String = match serde_json::to_string(&msg) {
//                         //                 Ok(s) => s,
//                         //                 Err(err) => {
//                         //                     eprintln!("{err}: Could not convert Message to String");
//                         //                     continue;
//                         //                 }
//                         //             };
//                         //             match ws_arc_c
//                         //                 .lock()
//                         //                 .unwrap()
//                         //                 .write_message(tungstenite::Message::Text(json))
//                         //             {
//                         //                 Ok(()) => (),
//                         //                 Err(err) => {
//                         //                     eprintln!("{err}: Error writing message");
//                         //                     continue;
//                         //                 }
//                         //             }
//                         //         }
//                         //         Err(e) => eprintln!("{e}: Error receiving data from main thread"),
//                         //     };
//                         // });
//                     }
//                     Err(err) => eprintln!("{err}: Failed connection"),
//                 };
//             }
//         });
//         Self { // _handle
// 	}
//     }
// }
