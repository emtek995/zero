use actix_web::{web, HttpResponse};
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;

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
    match insert_subscriber(&db_client, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, db_client)
)]
pub async fn insert_subscriber(
    db_client: &mongodb::Client,
    form: &FormData,
) -> Result<(), mongodb::error::Error> {
    let db_options = UpdateOptions::builder().upsert(true).build();
    db_client
        .database("zero")
        .collection::<FormData>("subscribers")
        .update_one(
            doc! {
                "email": form.email.clone(),
            },
            doc! {
                "$set": {
                    "email": form.email.clone(),
                    "name": form.name.clone(),
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
