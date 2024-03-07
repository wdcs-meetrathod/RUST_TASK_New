use serde::{ Deserialize, Serialize };
use tungstenite::connect;
use url::Url;
use std::fs::{ self, OpenOptions };
use std::io::Write;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct BinanceResult {
    e: String,
    E: u64,
    s: String,
    t: u64,
    p: String,
    q: String,
    b: u64,
    a: u64,
    T: u64,
    m: bool,
    M: bool,
}

pub fn cache_price(time: u32) -> Result<f64, String> {
    let binance_url = format!("{}/ws", "wss://stream.binance.com:9443/ws/btcusdt@trade");

    let (mut socket, _response) = connect(Url::parse(&binance_url).unwrap()).expect(
        "Could't connect to websocket"
    );

    let mut price: f64 = 0.0;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./priceDetails.txt")
        .expect("Error while opening file");

    let start_time = Instant::now();

    while start_time.elapsed().as_secs() < time.into() {
        let msg = socket.read_message().expect("Error reading message");

        let msg = match msg {
            tungstenite::Message::Text(s) => s,

            tungstenite::Message::Binary(_) => {
                let response = format!("Received binary");
                println!("{response}");
                return Err(response);
            }
            tungstenite::Message::Ping(_) => {
                let response = format!("Received ping");
                println!("{response}");
                return Err(response);
            }
            tungstenite::Message::Pong(_) => {
                let response = format!("Received Pong");
                println!("{response}");
                return Err(response);
            }
            tungstenite::Message::Close(_) => {
                let response = format!("Received Disconnected");
                println!("{response}");
                return Err(response);
            }
        };

        let data: BinanceResult = serde_json::from_str(&msg).expect("Unable to parse message");

        let btc_price = data.p;

        let price_value = match btc_price {
            s => {
                match s.parse::<f64>() {
                    Ok(p) => p,
                    Err(_) => {
                        eprintln!("Unable to parse 'p' field as f64: {}", s);
                        continue;
                    }
                }
            }
            _ => {
                eprintln!("'p' field is not a string: {:?}", btc_price);
                continue;
            }
        };

        writeln!(file, "{}", price_value).expect("Error while writing in file");

        price += price_value;
    }

    let result = price / (time as f64);
    let response = format!("The average USD price of BTC is: {:?}", result);

    writeln!(file, "{}", response).expect("Error while writing in file");

    // println!("Cache complete. {}", response);

    Ok(result)
}
