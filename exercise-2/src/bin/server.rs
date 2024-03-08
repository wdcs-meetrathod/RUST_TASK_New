use std::{ error::Error, fs::{ File, OpenOptions }, net::SocketAddr, sync::Arc };
use std::io::Write;
use tokio::{ io::{ AsyncReadExt, AsyncWriteExt, BufReader }, net::TcpListener, sync::Mutex };

struct SharedState {
    prices: Vec<f64>,
    connected_clients: usize,
}

async fn handle_connection(
    add: SocketAddr,
    mut ws_stream: tokio::sync::MutexGuard<'_, tokio::net::TcpStream>,
    shared_state: Arc<Mutex<SharedState>>,
    text: String,
    mut file: File
) -> Result<(), Box<dyn Error + Send + Sync>> {
    const TOTAL_CLIENT: usize = 5;

    ws_stream.write_all("Welcome to the aggregator".as_bytes()).await.expect("Handshake failed");

    let mut guard = shared_state.lock().await;
    guard.connected_clients += 1;

    println!("From client {add:?} {text:?}");

    if let Ok(price) = text.parse::<f64>() {
        guard.prices.push(price);
        let result = format!("From client {}---> {price}", guard.connected_clients);
        writeln!(file, "{}", result).expect("Error while writing in file");
    }

    if guard.prices.len() == TOTAL_CLIENT {
        let sum: f64 = guard.prices.iter().sum();
        let final_aggregated_price = sum / (TOTAL_CLIENT as f64);

        let result = format!("Final aggregated price {final_aggregated_price} ");
        println!("{}", result);

        writeln!(file, "{}", result).expect("Error while writing in file");
        std::process::exit(0);
    }

    drop(guard);

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
        })
    );

    println!("Server in listing on {address}");

    loop {
        let (socket, add) = listener.accept().await.expect("Socket failed");

        println!("New connection {add}");

        // let file_path = "./clientAggregationData.txt";
        // let file_exists = std::path::Path::new(file_path).exists();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            // .truncate(!file_exists)
            .open("./clientAggregationData.txt")
            .expect("Error while opening file");

        let shared_state_clone = Arc::clone(&shared_state); // Clone for each connection

        tokio::spawn(async move {
            let socket = Arc::new(Mutex::new(socket));
            let mut socket_guard = socket.lock().await;

            let mut reader = BufReader::new(&mut *socket_guard);
            let mut buffer = [0_u8; 1024];

            match reader.read(&mut buffer).await {
                Ok(read_size) if read_size != 0 => {
                    let data = buffer[0..read_size].to_vec();
                    let string_data = String::from_utf8_lossy(&data).to_string();

                    handle_connection(
                        add,
                        socket_guard,
                        shared_state_clone,
                        string_data,
                        file
                    ).await
                }

                Ok(_) => {
                    eprintln!("No data from {}", add);
                    Ok(())
                }

                Err(err) => {
                    eprintln!("Error reading from {} {}", add, err);
                    Err(err.into())
                }
            }
        });
    }
}
