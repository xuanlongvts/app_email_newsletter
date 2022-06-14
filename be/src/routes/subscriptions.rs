use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{SubscriberName, NewSubscriber, SubscriberEmail};

#[derive(serde::Deserialize)]
pub struct FormData {
	email: String,
	name: String,
}

#[tracing::instrument(
	name= "Adding a new subscriber",
	skip(form, pool),
	fields(
		subscriber_email = %form.email,
		subscriber_name = %form.name
	)
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        // Return early if the name is invalid, with a 400
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    let email = match SubscriberEmail::parse(form.0.email) {
        Ok(email) => email,
        Err(_) => return HttpResponse::BadRequest().finish()
    };
    let new_subscriber = NewSubscriber {
        email,
        name
    };

	match insert_subscriber(&pool, &new_subscriber).await {
		Ok(_) => HttpResponse::Ok().finish(),
		Err(_) => HttpResponse::InternalServerError().finish(),
	}
}

#[tracing::instrument(
	name = "Saving new subscriber details in the database",
	skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
	let id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());
	sqlx::query!(
		r#"
		INSERT INTO subscriptions (id, email, name, subscribed_at)
		VALUES ($1, $2, $3, $4)
		"#,
		id,
		new_subscriber.email.as_ref(),
		new_subscriber.name.as_ref(),
		Utc::now()
	)
	.execute(pool)
	.await
	.map_err(|e| {
		tracing::error!("Failed to execute query: {:?}", e);
		e
	})?;

	Ok(())
}
