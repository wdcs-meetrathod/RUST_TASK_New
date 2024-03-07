use std::{ error::Error, net::SocketAddr, sync::Arc };
use futures_util::{ SinkExt, StreamExt };
use rand_core::OsRng;
use tokio::{ io::AsyncReadExt, io::BufReader, net::{ TcpListener, TcpStream }, sync::Mutex };
use tokio_websockets::{ Message, ServerBuilder, WebSocketStream };
use ed25519_dalek::Keypair;

struct SharedState {
    prices: Vec<f64>,
    token: Keypair,
    connected_clients: usize,
}

async fn handle_connection(
    add: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    shared_state: Arc<Mutex<SharedState>>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    const TOTAL_CLIENT: usize = 5;

    ws_stream
        .send(Message::text("Welcome to the aggregator".to_string())).await
        .expect("Handshake failed");

    loop {
        let mut guard = shared_state.lock().await;

        guard.connected_clients += 1;

        let incoming = match ws_stream.next().await {
            Some(Ok(message)) => message,
            Some(Err(err)) => {
                println!("Error while receiving the message {}", err);
                return Err(err.into());
            }
            None => {
                break;
            }
        };

        if let Some(text) = incoming.as_text() {
            println!("From client {add:?} {text:?}");

            if let Ok(price) = text.parse::<f64>() {
                guard.prices.push(price);
            }
        }
    }

    let guard = shared_state.lock().await;
    if guard.prices.len() == TOTAL_CLIENT {
        let sum: f64 = guard.prices.iter().sum();
        let final_aggregated_price = sum / (TOTAL_CLIENT as f64);
        println!("Final aggregated price {final_aggregated_price} ");
        std::process::exit(0);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let address = "127.0.0.1:8080";

    let listener = TcpListener::bind(address).await.expect("Failed to connect");

    let shared_state = Arc::new(
        Mutex::new(SharedState {
            connected_clients: 0,
            prices: Vec::new(),
            token: Keypair::generate(&mut OsRng),
        })
    );

    println!("Server in listing on {address}");

    loop {
        let (mut socket, add) = listener.accept().await.expect("Socket failed");

        println!("New connection {add}");

        let shared_state_clone = Arc::clone(&shared_state); // Clone for each connection

        tokio::spawn(async move {
            let mut reader = BufReader::new(socket);
            let mut buffer = [0_u8; 1024];
            let read_size = reader.read(&mut buffer).await.expect("Unable to read");
            if read_size != 0 {
                let data = buffer[0..read_size].to_vec();
                let string_data = String::from_utf8_lossy(&data).to_string();

                println!("{string_data}");
            }
        });

        // let mut recv_str = String::new();
        // let msg_size = socket.read_to_string(&mut recv_str).await.expect("unable to read");
        // println!("read_string: {:?}", recv_str);
        // if msg_size == 0 {
        //     break;
        // }

        // tokio::spawn(async move {
        //     let ws_steam = ServerBuilder::new().accept(socket).await.expect("Server build failed");

        //
        // });
    }
}
