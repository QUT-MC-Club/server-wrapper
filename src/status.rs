pub mod webhook;

#[derive(Clone)]
pub struct StatusWriter {
    webhook: Option<webhook::Client>,
}

impl StatusWriter {
    pub fn none() -> StatusWriter {
        StatusWriter { webhook: None }
    }

    pub fn write(&self, message: impl Into<String>) {
        if let Some(webhook) = &self.webhook {
            let webhook = webhook.clone();
            let message = message.into();

            tokio::spawn(async move {
                let payload = webhook::Payload::new_sanitized(message);
                let result = webhook.post(&payload).await;

                if let Err(err) = result {
                    eprintln!("failed to post to webhook: {:?}", err);
                }
            });
        }
    }
}

impl From<webhook::Client> for StatusWriter {
    fn from(webhook: webhook::Client) -> Self {
        StatusWriter {
            webhook: Some(webhook)
        }
    }
}
