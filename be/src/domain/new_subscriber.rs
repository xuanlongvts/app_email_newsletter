use crate::domain::subscriber_name::SubscriberName;
use crate::domain::subscriber_email::SubscriberEmail;

pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail
}
