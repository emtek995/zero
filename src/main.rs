use std::net::TcpListener;

use actix_web::web;
use mongodb::{bson::doc, options::IndexOptions, IndexModel};
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

    let db_client =
        mongodb::Client::with_uri_str(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed connection to database");

    create_email_index(&db_client).await;

    let db_client = web::Data::new(db_client);
    run(listener, db_client)?.await?;
    Ok(())
}

async fn create_email_index(db_client: &mongodb::Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(options)
        .build();
    db_client
        .database("zero")
        .collection::<zero::routes::FormData>("subscribers")
        .create_index(model, None)
        .await
        .expect("Failed to create index");
}
