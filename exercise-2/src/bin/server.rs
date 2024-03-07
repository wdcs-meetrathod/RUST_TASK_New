use std::{ error::Error, net::SocketAddr, sync::Arc };
use futures_util::{ SinkExt, StreamExt };
use tokio::{ net::{ TcpListener, TcpStream }, sync::Mutex };
use tokio_websockets::{ Message, ServerBuilder, WebSocketStream };

async fn handle_connection(
    add: SocketAddr,
    mut ws_steam: WebSocketStream<TcpStream>,
    client_averages: Arc<Mutex<Vec<f64>>>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    const TOTAL_CLIENT: usize = 5;

    ws_steam
        .send(Message::text("Welcome to the aggregator".to_string())).await
        .expect("Handshake failed");

    loop {
        tokio::select! {
        incoming = ws_steam.next()=>{
            match incoming{
                Some(Ok(message)) =>{
                    if let Some(text) = message.as_text(){
                          println!("From client {add:?} {text:?}");
                            let mut guard = client_averages.lock().await;
                          if let Ok(price) = text.parse::<f64>(){
                            guard.push(price);
                          };

                    };
                   }

                   Some(Err(err))=>{
                    return Err(err.into());
                   }
                   None =>{
                    break;
                   }
                }
            }

        }
    }

    println!("{}", client_averages.lock().await.len());

    if client_averages.lock().await.len() == TOTAL_CLIENT {
        let sum: f64 = client_averages.lock().await.iter().sum();
        let final_aggregated_prise = sum / (TOTAL_CLIENT as f64);
        println!("Final aggregated prise {final_aggregated_prise} ");
        std::process::exit(1);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let address = "127.0.0.1:8080";

    let listener = TcpListener::bind(address).await.expect("Failed to connect");

    let client_averages: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));

    println!("Server in listing on {address}");

    loop {
        let (socket, add) = listener.accept().await.expect("Socket failed");

        println!("New connection {add}");

        let client_averages_clone = Arc::clone(&client_averages); // Clone for each connection

        tokio::spawn(async move {
            let ws_steam = ServerBuilder::new().accept(socket).await.expect("Server build failed");

            handle_connection(add, ws_steam, client_averages_clone).await
        });
    }
}
