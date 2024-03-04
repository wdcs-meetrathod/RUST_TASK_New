pub mod handle {
    use std::{ net::SocketAddr, sync::Arc };
    use data_encoding::BASE64;
    use futures_util::SinkExt;
    use tokio::net::TcpStream;
    use tokio::sync::Mutex;
    use tokio_tungstenite::{ accept_async, tungstenite::protocol::Message, WebSocketStream };
    use futures_util::stream::{ SplitSink, SplitStream, StreamExt };
    use ed25519_dalek::{ Keypair, Signer, Verifier, Signature };

    pub struct SharedState {
        pub prices: Vec<f64>,
        pub token: Keypair,
        pub connected_clients: usize,
    }

    fn calculate_average_price(prices: &[f64]) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }

        let sum: f64 = prices.iter().sum();
        sum / (prices.len() as f64)
    }

    fn sign_message(keypair: &Keypair, message: &str) -> String {
        let signature = keypair.sign(message.as_bytes());
        BASE64.encode(signature.as_ref())
    }

    pub async fn handle_connection(
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

        drop(guard);
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

                        let message = text.as_str();

                        // Sign the message
                        let signature = sign_message(&guard.token, message);

                        // Decode signature
                        let signature_bytes = match BASE64.decode(signature.as_bytes()) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                eprintln!("Failed to decode signature from base64: {}", e);
                                continue;
                            }
                        };

                        // println!("Decoded Signature Bytes Length: {}", signature_bytes.len());

                        // Create signature from bytes
                        let signature1 = match Signature::from_bytes(&signature_bytes) {
                            Ok(sig) => sig,
                            Err(e) => {
                                eprintln!("Failed to create signature from bytes: {}", e);
                                continue;
                            }
                        };

                        // Verify the signature
                        if
                            guard.token.public
                                .verify(text.to_string().as_bytes(), &signature1)
                                .is_ok()
                        {
                            println!("Valid message");
                            guard.prices.push(price);

                            if
                                guard.connected_clients == CLIENT_LIMIT &&
                                guard.prices.len() == CLIENT_LIMIT
                            {
                                let average_price = calculate_average_price(&guard.prices);
                                println!("Average Price: {}", average_price);

                                let response = format!("Average Price: {}", average_price);

                                let _ = write.send(Message::text(response)).await;

                                // Clear prices for the next calculation
                                guard.prices.clear();
                                guard.connected_clients = 0;
                            } else {
                                println!("Waiting for remaining clients to enter the price {}", add);
                                let _ = write.send(Message::text("thanks")).await;
                            }
                        } else {
                            // Throw the error message
                            let _ = write.send(Message::text("Unauthorized")).await;
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
}
