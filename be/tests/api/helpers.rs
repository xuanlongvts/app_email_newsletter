use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use email_newsletter::configuration::{get_configuration, DatabaseSettings};
use email_newsletter::startup::{get_connection_pool, Application};
use email_newsletter::telemetry::{get_subscriber, init_subscriber};
use linkify::{LinkFinder, LinkKind};
use once_cell::sync::Lazy;
use reqwest::Url;
use sha3::Digest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
}

/// TestUser
pub struct TestUser {
    pub user_id: sqlx::types::Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> TestUser {
        Self {
            user_id: sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128()),
            username: sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128()).to_string(),
            password: sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128()).to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash) VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
    }
}

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: Url,
    pub plain_text: Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_links = |s: &str| {
            let links: Vec<_> = LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();

            confirmation_link
        };
        let html = get_links(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_links(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletter(&self, body: serde_json::Value) -> reqwest::Response {
        // let (username, password) = self.test_user().await;
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // pub async fn test_user(&self) -> (String, String) {
    //     let row = sqlx::query!("SELECT username, password FROM users LIMIT 1")
    //         .fetch_one(&self.db_pool)
    //         .await
    //         .expect("Failed to create test users.");
    //     (row.username, row.password)
    // }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let app_port = application.port();
    let address = format!("http://127.0.0.1:{}", app_port);
    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        port: app_port,
        test_user: TestUser::generate(),
    };
    test_app.test_user.store(&test_app.db_pool).await;
    // add_test_app(&test_app.db_pool).await;
    test_app
}

// async fn add_test_app(pool: &PgPool) {
//     let user_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());
//     let username = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128()).to_string();
//     let password = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128()).to_string();
//     sqlx::query!(
//         "INSERT INTO users (user_id, username, password) VALUES ($1, $2, $3)",
//         user_id,
//         username,
//         password
//     )
//     .execute(pool)
//     .await
//     .expect("Failed to create test users.");
// }

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to create database.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to create database.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
