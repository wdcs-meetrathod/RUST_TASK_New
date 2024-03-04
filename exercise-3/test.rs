
extern crate rand_core;
extern crate ed25519_dalek;
extern crate hex;

use ed25519_dalek::*;


use rand_core::OsRng;
use std::env;


fn main() {


  let mut str1 ="This is a test of the Ed25519 signature in Rust.";


  let args: Vec = env::args().collect();
  
  if args.len() >1 { str1 = args[1].as_str();}


let keypair: Keypair = Keypair::generate(&mut OsRng);


println!("Secret key: {}",hex::encode(keypair.public.as_bytes()));
println!("Public key: {}",hex::encode(keypair.secret.as_bytes()));

let message: &[u8] = str1.as_bytes();

let signature: Signature = keypair.sign(message);

println!("Message: {}",str1);
println!("Signature: {}",signature.to_string());

let rtn=keypair.verify(message, &signature).is_ok();

if rtn==true {
  println!("Signature has been proven!");
}
else {
  println!("Signature has NOT been proven!");
}

}



use std::{ net::SocketAddr, sync::Arc };
use data_encoding::BASE64;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio::{ net::TcpListener, sync::Mutex };
use rand_core::OsRng;
use tokio_tungstenite::accept_async;
mod utils;
use tokio_tungstenite::{ tungstenite::protocol::Message, WebSocketStream };
use futures_util::stream::{ SplitSink, SplitStream, StreamExt };
use ed25519_dalek::{ Keypair, Signer, Verifier, Signature };
use utils::base::SharedState;

use crate::utils::base::{ calculate_average_price, parse_price, sign_message };

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    println!("WebSocket server is running on: {}", addr);

    // Shared state for storing prices and counting connected clients
    let shared_state = Arc::new(
        Mutex::new(utils::base::SharedState {
            prices: Vec::new(),
            token: Keypair::generate(&mut OsRng),
            connected_clients: 0,
        })
    );

    while let Ok((stream, add)) = listener.accept().await {
        let shared_state_clone = Arc::clone(&shared_state);
        println!("here");
        tokio::spawn(handle_connection(stream, add, shared_state_clone)).await.expect("Error");
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    add: SocketAddr,
    shared_state: Arc<Mutex<SharedState>>
) {
    let ws_stream = accept_async(stream).await.expect("Error during WebSocket handshake");

    println!("WebSocket connection established for client {}", add);

    let mut guard = shared_state.lock().await;

    guard.connected_clients += 1;

    let (write, read) = ws_stream.split();

    tokio::spawn(read_from_client(read, write, add, shared_state.clone())); //.expect("Error");

    // tokio::spawn(write_to_client(write));

    drop(guard);
}

pub async fn read_from_client(
    mut read: SplitStream<WebSocketStream<tokio::net::TcpStream>>,
    mut write: SplitSink<WebSocketStream<TcpStream>, Message>,
    add: SocketAddr,
    shared_state: Arc<Mutex<SharedState>>
) {
    println!("read_from_client");
    const CLIENT_LIMIT: usize = 5;

    let mut guard = shared_state.lock().await;
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                let message = text.as_str();

                println!("Message {}", message);

                //Sign the message
                // let signature = sign_message(&guard.token, message);

                // // Decode signature
                // let signature_bytes = match BASE64.decode(signature.as_bytes()) {
                //     Ok(bytes) => bytes,
                //     Err(e) => {
                //         eprintln!("Failed to decode signature from base64: {}", e);
                //         continue;
                //     }
                // };

                // // println!("Decoded Signature Bytes Length: {}", signature_bytes.len());

                // // Create signature from bytes
                // let signature1 = match Signature::from_bytes(&signature_bytes) {
                //     Ok(sig) => sig,
                //     Err(e) => {
                //         eprintln!("Failed to create signature from bytes: {}", e);
                //         continue;
                //     }
                // };

                // if guard.token.public.verify(text.to_string().as_bytes(), &signature1).is_ok() {
                println!("Signature is valid for client {} {}", message, add);

                if let Ok(price) = parse_price(message) {
                    //Push price in shared state
                    guard.prices.push(price);

                    if
                        guard.connected_clients == CLIENT_LIMIT &&
                        guard.prices.len() == CLIENT_LIMIT
                    {
                        let average_price = calculate_average_price(&guard.prices);

                        println!("Average Price: {}", average_price);

                        let response_message = format!("Average Price: {}", average_price);

                        //Respond to the client
                        // write.send(Message::text(response_message));

                        // Clear prices for the next calculation
                        guard.prices.clear();
                        guard.connected_clients = 0;
                    } else {
                        let response_message = "Waiting for remaining clients to enter the price";

                        println!("{} {}", response_message, add);

                        // write.send(Message::text(response_message)).await;
                    }
                }
                // } else {
                //     println!("Signature is invalid for client {} ", add);
                // }
            }
            Message::Binary(bin) => {
                println!("Received binary data: {:?}", bin);
            }
            Message::Close(_) => {
                println!("WebSocket closed by client {}", add);
                break;
            }
            _ => {}
        }
    }
}
