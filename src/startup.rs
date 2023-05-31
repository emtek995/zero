use std::net::TcpListener;

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use anyhow::Result;
use tracing_actix_web::TracingLogger;

use crate::{email_client::EmailClient, routes::*};

pub fn run(
    listener: TcpListener,
    db_client: web::Data<mongodb::Client>,
    email_client: EmailClient,
) -> Result<Server> {
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_client.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
