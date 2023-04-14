use std::net::TcpListener;

use mongodb::bson::doc;
use zero::configuration::get_configuration;

fn spawn_app() -> String {
    let listener = TcpListener::bind("localhost:0").expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();
    let server = zero::startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://localhost:{port}")
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    use zero::routes::FormData;

    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection = mongodb::Client::with_uri_str(&connection_string)
        .await
        .expect("Failed to connect to Mongodb");
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{app_address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = connection
        .database("zero")
        .collection::<FormData>("subscriptions")
        .find_one(doc! {}, None)
        .await
        .expect("Failed to get subscriptions")
        .unwrap();

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{app_address}/subscriptions"))
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
