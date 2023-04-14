use std::net::TcpListener;

use zero::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to get configuration");
    let listener = TcpListener::bind(format!("localhost:{}", configuration.application_port))
        .expect("Failed to bind port");
    run(listener)?.await
}
