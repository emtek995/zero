use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::routes::*;

pub fn run(
    listener: TcpListener,
    db_client: web::Data<mongodb::Client>,
) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
