use actix_web::web;
use mongodb::bson::doc;
use secrecy::ExposeSecret;
use std::net::TcpListener;
use std::sync::Once;
use zero::{
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Once = Once::new();

pub struct TestApp {
    pub address: String,
    pub db_client: web::Data<mongodb::Client>,
}

async fn spawn_app() -> TestApp {
    TRACING.call_once(|| {
        let default_filter_level = "info".into();
        let subscriber_name = "test".into();
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });

    let listener = TcpListener::bind("localhost:0").expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://localhost:{port}");

    let configuration = get_configuration().expect("Failed to read configuration");
    let db_client =
        mongodb::Client::with_uri_str(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Mongodb");
    let db_client = web::Data::new(db_client);

    let server = zero::startup::run(listener, db_client.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address, db_client }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    use zero::routes::FormData;

    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    if let Some(saved) = app
        .db_client
        .database("zero")
        .collection::<FormData>("subscriptions")
        .find_one(doc! {"email": "ursula_le_guin@gmail.com"}, None)
        .await
        .expect("Failed to get subscriptions")
    {
        assert_eq!(saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(saved.name, "le guin");
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute requst");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Requst when the payload was {error_message}"
        );
    }
}
