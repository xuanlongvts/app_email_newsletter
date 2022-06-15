use crate::domain::SubscriberEmail;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: Client
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender
        }
    }

    pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), String> {
        todo!()
    }
}
