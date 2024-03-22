use ed25519_dalek::Keypair;
use rand_core::OsRng;

fn main() {
    #[derive(Debug)]
    struct ClientKeys {
        client: u32,
        private_key: String,
        public_key: String,
    }

    let mut tokens: Vec<ClientKeys> = Vec::new();

    for i in 0..5 {
        let token = Keypair::generate(&mut OsRng);
        let client_key = ClientKeys {
            client: i + 1,
            private_key: hex::encode(token.secret.as_ref()),
            public_key: hex::encode(token.public.as_ref()),
        };

        tokens.push(client_key);
    }
    println!("{:?}", tokens);
}
