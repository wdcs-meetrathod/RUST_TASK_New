use std::time::Duration;
use tokio::{ io::AsyncWriteExt, net::TcpStream, time::sleep };

#[path = "../my_module.rs"]
mod my_module;

async fn send_to_aggregate(average: f64) {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.expect("Error while connecting");

    stream.write_all(average.to_string().as_bytes()).await.expect("error while sending message");
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
