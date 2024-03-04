use tungstenite::connect;
use url::Url;
use std::{ env, fs::{ self, OpenOptions } };
use std::io::Write;

fn auth_mode(s: &str) -> String {
    let s = s.to_lowercase();
    if s == "cache" || s == "read" {
        String::from(s)
    } else {
        panic!("Error: Mode value must be cache | read ")
    }
}

fn auth_time(time: &str) -> usize {
    match time.parse::<usize>() {
        Ok(value) => value,
        Err(_) => panic!("Error: Time must be a valid number"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mode = &args[1].split('=').nth(1).unwrap();

    let mode = auth_mode(&mode);

    if mode == "cache" {
        let time = &args[2].split('=').nth(1).unwrap();

        let time = auth_time(&time);

        let total = time as f64;

        let binance_url = format!("{}/ws", "wss://stream.binance.com:9443/ws/btcusdt@trade");

        let (mut socket, _response) = connect(Url::parse(&binance_url).unwrap()).expect(
            "Could't connect to websocket"
        );

        let mut price: f64 = 0.0;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("./data.txt")
            .expect("Error while opening file");

        for _ in 0..time {
            // println!("{}", i);
            let msg = socket.read_message().expect("Error reading message");

            let msg = match msg {
                tungstenite::Message::Text(s) => s,
                _ => { panic!("Error getting text") }
            };

            let data: serde_json::Value = serde_json
                ::from_str(&msg)
                .expect("Unable to parse message");

            let btc_price = data.get("p").expect("Could't get the price of BTC");

            let price_value = match btc_price {
                serde_json::Value::String(s) => {
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

        let result = price / total;

        println!("Cache complete. The average USD price of BTC is: {:?}", result);
    } else {
        let contents = fs
            ::read_to_string("./data.txt")
            .expect("Should have been able to read the file");

        println!("USD prices of BTC:\n{contents}");
    }
}
