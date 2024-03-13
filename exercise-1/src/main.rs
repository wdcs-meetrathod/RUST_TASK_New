use clap::Parser;
mod my_module;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long)]
    mode: String,

    #[clap(short, long)]
    time: Option<String>,
}

fn auth_mode(mode: &str) -> Result<&str, String> {
    match mode {
        "cache" | "read" => Ok(mode),
        _ => {
            let error_message = format!("Invalid mode {}", mode);
            Err(error_message)
        }
    }
}

fn parse_time(time: &Option<String>) -> Result<u32, &str> {
    match time {
        Some(t) =>
            match t.parse::<u32>() {
                Ok(value) => Ok(value),
                Err(_) => Err("Error: Time must be a valid number"),
            }
        None => { Err("Error: When 'cache' mode is specified, 'time' is required.") }
    }
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    if args.mode == "cache" && args.time.is_none() {
        eprintln!("Error: When 'cache' mode is specified, 'time' is required.");
        std::process::exit(1);
    }

    match auth_mode(args.mode.as_str()) {
        Ok(mode) => {
            if mode == "cache" {
                let time = match parse_time(&args.time) {
                    Ok(t) => t,
                    Err(err) => {
                        eprintln!("{}", err);
                        std::process::exit(1)
                    }
                };
                my_module::cache_price(time)
            } else {
                my_module::read_price()
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
    };
}
