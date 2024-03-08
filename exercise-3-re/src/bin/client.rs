use std::time::Duration;
use tokio::{ io::AsyncWriteExt, net::TcpStream, time::sleep };

#[path = "../my_module.rs"]
mod my_module;

async fn send_to_aggregate((client, average): (i32, f64)) {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.expect("Error while connecting");
    let signature = my_module::sign_message(client, average.to_string());

    let updated = &signature.to_string();

    // println!("{updated}");

    let serialized_message = bincode::serialize(&(average.to_string(), &updated)).unwrap();

    stream.write_all(&serialized_message).await.expect("error while sending message");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..5 {
        match my_module::cache_price(2) {
            Ok(average) => {
                println!("Average of client {i}--> {average}");
                send_to_aggregate((i + 1, average)).await;
            }
            Err(err) => eprintln!("{err}"),
        }
        sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
