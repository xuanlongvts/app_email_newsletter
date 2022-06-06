use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
	email: String,
	name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
	let id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());
	match sqlx::query!(
		r#"
		INSERT INTO subscriptions (id, email, name, subscribed_at)
		VALUES ($1, $2, $3, $4)
		"#,
		id,
		form.email,
		form.name,
		Utc::now()
	)
	.execute(pool.get_ref())
	.await
	{
		Ok(_) => HttpResponse::Ok().finish(),
		Err(e) => {
			println!("Failed to execute query: {}", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}
