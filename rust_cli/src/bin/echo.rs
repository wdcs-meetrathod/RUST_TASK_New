use std::{
    env,
    io::{self, Write},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usages <message>")
    }
    let message = format!("{}", &args[1..].join(" "));

    let _ = io::stdout().write_all(message.as_bytes());
}
