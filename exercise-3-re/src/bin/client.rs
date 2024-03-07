use std::time::Duration;
use futures_util::SinkExt;
use http::Uri;
use rand_core::OsRng;
use tokio::{ io::AsyncWriteExt, net::TcpStream, time::sleep };
use tokio_websockets::{ ClientBuilder, Message };
use ed25519_dalek::{ ed25519::signature::SignerMut, Keypair };

#[path = "../my_module.rs"]
mod my_module;

async fn send_to_aggregate(average: f64) {
    let mut stream =
        TcpStream::connect("ws://127.0.0.1:8080").await.expect("Error while connecting");

    stream.write_all(average.to_string().as_bytes()).await.expect("error while sending message");
    // let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
    //     .connect().await
    //     .expect("Network connection error");

    // let mut token = Keypair::generate(&mut OsRng);

    // let sign_message = token.sign(average.to_string().as_bytes());

    ()

    // ws_stream
    //     .send(Message::text(sign_message.to_string())).await
    //     // .send(Message::text(average.to_string())).await
    //     .expect("Error while sending to the aggregate");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..5 {
        match my_module::cache_price(2) {
            Ok(average) => {
                println!("Average of client {i}--> {average}");
                send_to_aggregate(average).await;
            }
            Err(err) => eprintln!("{err}"),
        }
        sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
