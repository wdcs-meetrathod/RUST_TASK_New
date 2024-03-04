use futures_util::stream::SplitSink;
use tokio::net::{ TcpListener, TcpStream };
use tokio_tungstenite::{ tungstenite::protocol::Message, accept_async, WebSocketStream };
use futures_util::stream::{ SplitStream, StreamExt };
use futures_util::sink::SinkExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8001";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    println!("WebSocket server is running on: {}", addr);

    // Shared state for storing prices and counting connected clients
    let shared_state = Arc::new(
        Mutex::new(SharedState {
            prices: Vec::new(),
            connected_clients: 0,
        })
    );

    while let Ok((stream, add)) = listener.accept().await {
        let shared_state_clone = Arc::clone(&shared_state);

        tokio::spawn(handle_connection(stream, add, shared_state_clone));
    }
}

struct SharedState {
    prices: Vec<f64>,
    connected_clients: usize,
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

    tokio::spawn(read_from_client(read, write, add, shared_state.clone()));

    // tokio::spawn(write_to_client(write));

    drop(guard);
}

fn calculate_average_price(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }

    let sum: f64 = prices.iter().sum();
    sum / (prices.len() as f64)
}

async fn read_from_client(
    mut read: SplitStream<WebSocketStream<tokio::net::TcpStream>>,
    mut write: SplitSink<WebSocketStream<TcpStream>, Message>,
    add: SocketAddr,
    shared_state: Arc<Mutex<SharedState>>
) {
    const CLIENT_LIMIT: usize = 5;
    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(price) = text.parse::<f64>() {
                    println!("Received price {} from client {}", price, add);

                    // Acquire a lock on the shared state to update prices
                    let mut guard = shared_state.lock().await;
                    guard.prices.push(price);

                    if
                        guard.connected_clients == CLIENT_LIMIT &&
                        guard.prices.len() == CLIENT_LIMIT
                    {
                        let average_price = calculate_average_price(&guard.prices);
                        let response = format!("Average Price: {}", average_price);
                        println!("{}", response);
                        let _ = write.send(Message::text(response));

                        // Clear prices for the next calculation
                        guard.prices.clear();
                        guard.connected_clients = 0;
                    } else {
                        println!("Waiting for remaining clients to enter the price {}", add);
                        let _ = write.send(Message::text("thanks")).await;
                    }
                } else {
                    eprintln!("Invalid price format from client {}", add);
                }
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
