#[tokio::main]
async fn main() {
    let port: u16 = match std::env::var("PORT") {
        Ok(val) => match val.parse() {
            Ok(port) => port,
            Err(_) => {
                eprintln!("invalid PORT value: {val:?}");
                std::process::exit(1);
            }
        },
        Err(_) => 8080,
    };

    let listener = match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("failed to bind 0.0.0.0:{port}: {e}");
            std::process::exit(1);
        }
    };

    match listener.local_addr() {
        Ok(addr) => println!("listening on {addr}"),
        Err(e) => {
            eprintln!("failed to read bound address: {e}");
            std::process::exit(1);
        }
    }

    if let Err(e) = axum::serve(listener, openrtb_validator::app()).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
