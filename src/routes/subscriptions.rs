use actix_web::{web, HttpResponse};
use anyhow::Result;
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_client: web::Data<mongodb::Client>,
) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name,
    };
    match insert_subscriber(&db_client, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, db_client)
)]
pub async fn insert_subscriber(
    db_client: &mongodb::Client,
    new_subscriber: &NewSubscriber,
) -> Result<()> {
    let db_options = UpdateOptions::builder().upsert(true).build();
    db_client
        .database("zero")
        .collection::<FormData>("subscribers")
        .update_one(
            doc! {
                "email": new_subscriber.email.clone(),
            },
            doc! {
                "$set": {
                    "email": new_subscriber.email.clone(),
                    "name": new_subscriber.name.as_ref(),
                }
            },
            Some(db_options),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}
