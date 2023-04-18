use std::net::TcpListener;

use actix_web::web;
use mongodb::{bson::doc, options::IndexOptions, Client, IndexModel};
use secrecy::ExposeSecret;

use zero::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to get configuration");

    let listener = TcpListener::bind(format!("localhost:{}", configuration.application_port))
        .expect("Failed to bind port");

    let connection =
        mongodb::Client::with_uri_str(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed connection to database");

    create_email_index(&connection).await;

    let connection = web::Data::new(connection);
    run(listener, connection)?.await?;
    Ok(())
}

async fn create_email_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(options)
        .build();
    client
        .database("zero")
        .collection::<zero::routes::FormData>("subscribers")
        .create_index(model, None)
        .await
        .expect("Failed to create index");
}
