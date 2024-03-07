use std::time::Duration;
use futures_util::SinkExt;
use http::Uri;
use tokio::time::sleep;
use tokio_websockets::{ ClientBuilder, Message };

#[path = "../my_module.rs"]
mod my_module;

async fn send_to_aggregate(average: f64) {
    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
        .connect().await
        .expect("Network connection error");

    ws_stream
        .send(Message::text(average.to_string())).await
        .expect("Error while sending to the aggregate");
    ()
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
