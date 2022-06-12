use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use email_newsletter::configuration::get_configuration;
use email_newsletter::startup::run;
use email_newsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber(
        "app_email_newsletter".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new().connect_timeout(std::time::Duration::from_secs(2)).connect_lazy_with(configuration.database.with_db());

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
