use std::time::Duration;
use tokio::{ io::AsyncWriteExt, net::TcpStream, time::sleep };

#[path = "../my_module.rs"]
mod my_module;

async fn send_to_aggregate((client, average): (i32, f64)) {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.expect("Error while connecting");

    let (message, signature) = my_module::sign_message(client, average.to_string());

    let serialized_message = bincode::serialize(&(message, signature.as_ref())).unwrap();

    stream.write_all(&serialized_message).await.expect("error while sending message");
}

async fn run_client(client_id: i32) {
    match my_module::cache_price(2) {
        Ok(average) => {
            println!("Average of client {client_id}--> {average}");
            send_to_aggregate((client_id, average)).await;
        }
        Err(err) => eprintln!("{err}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        for i in 0..5 {
            run_client(i + 1).await;
        }
        sleep(Duration::from_secs(5)).await;
    }
}
