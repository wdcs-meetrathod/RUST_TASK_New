use ed25519_dalek::{ ed25519::Error, Keypair, Signature, Signer };
use serde::{ Deserialize, Serialize };
use std::fmt::Debug;
use std::fs::{ self, OpenOptions };
use std::io::Write;
use std::time::Instant;
use tungstenite::connect;
use url::Url;

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

pub struct ClientKeys {
    pub client: i32,
    pub private_key: String,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    pub message: String,
    pub client_id: i32,
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
            s =>
                match s.parse::<f64>() {
                    Ok(p) => p,
                    Err(_) => {
                        eprintln!("Unable to parse 'p' field as f64: {}", s);
                        continue;
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

pub fn read_price() {
    let contents = fs
        ::read_to_string("./priceDetails.txt")
        .expect("Should have been able to read the file");

    println!("USD prices of BTC:\n{contents}");
}

pub fn get_keys(client: i32) -> Option<ClientKeys> {
    let client_keys: Vec<ClientKeys> = vec![
        ClientKeys {
            client: 1,
            private_key: "d367b3e44029bcb967463a030f4d7912f754fdf3df7e95094faf7ae20bfbb1d6".to_string(),
            public_key: "5dd304258e3bcc9d43a1773ecfac51028ce180028502c4d2929adc9a9daa9b57".to_string(),
        },
        ClientKeys {
            client: 2,
            private_key: "60aeb5c892f8983def0dce3dceb902af32bd47264e5f76bb0c48cc723de8afbc".to_string(),
            public_key: "2e857830d934b1d43dffcf8c97f7790045c77518a3b405244e84a2e9199f0997".to_string(),
        },
        ClientKeys {
            client: 3,
            private_key: "9983fffa551801060638fb4ea8968fa879768d1d9cd8d8c22f9a53d6581253c7".to_string(),
            public_key: "9831cdf3d299de2eea35140ade28c9fc6e0858837affc5d56317208e2deb30c4".to_string(),
        },
        ClientKeys {
            client: 4,
            private_key: "e3a1e9af3dce9ada31a4b0ddbc31818909efa6081af52dc112d0db87691c750f".to_string(),
            public_key: "417f173230605ba5e89a98bad1a04a3d3ca6ba25b68423b8ffd0e431d738db0c".to_string(),
        },
        ClientKeys {
            client: 5,
            private_key: "970fefe9139a781d7bad9954bcdc1ac841aa52309dee65b874491b01561bafdc".to_string(),
            public_key: "7bdfe218f62e5d472c1d5192966896a374d0a81cd4521d0fed6a5f0e930a0800".to_string(),
        }
    ];

    client_keys.into_iter().find(|key| key.client == client)
}

fn get_key_pair(client: i32) -> Keypair {
    if let Some(client_key) = get_keys(client) {
        let private_key_bytes = hex
            ::decode(&client_key.private_key)
            .expect("Invalid private key hex string");

        let public_key_bytes = hex
            ::decode(&client_key.public_key)
            .expect("Invalid public key hex string");

        let key_pair = Keypair {
            public: ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap(),

            secret: ed25519_dalek::SecretKey::from_bytes(&private_key_bytes).unwrap(),
        };

        key_pair
    } else {
        eprintln!("Key not found for {client}");
        std::process::exit(1);
    }
}

pub fn sign_message(client: i32, message: String) -> (String, Signature) {
    let key_pair = get_key_pair(client);
    let message = ClientMessage {
        message,
        client_id: client,
    };

    let serialized_message = serde_json::to_string(&message).expect("Can serialize");

    let signature = key_pair.sign(serialized_message.as_ref());

    (serialized_message, signature)
}

pub fn verify_message((message, signature): (String, &[u8])) -> Option<ClientMessage> {
    let client_message: ClientMessage = serde_json::from_str(&message).expect("can deserialize");

    let key_pair = get_key_pair(client_message.client_id);

    let signature: Result<Signature, Error> = match Signature::from_bytes(&signature) {
        Ok(signature) => Ok(signature),
        Err(err) => {
            eprintln!("{err}");
            Err(err)
        }
    };

    let auth = key_pair.public.verify_strict(
        &message.as_bytes(),
        &signature.expect("Invalid signature pattern")
    );

    if auth.is_ok() {
        Some(client_message)
    } else {
        eprintln!("Invalid signature from client {}", client_message.client_id);
        None
    }
}
