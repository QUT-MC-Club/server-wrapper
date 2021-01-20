use serde::Serialize;

#[derive(Serialize)]
pub struct Payload {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
}

impl Payload {
    pub fn new_sanitized(content: String) -> Payload {
        Payload {
            content,
            username: None,
            avatar_url: None,
            allowed_mentions: Some(AllowedMentions::sanitized()),
        }
    }
}

#[derive(Serialize)]
pub struct AllowedMentions {
    pub parse: Vec<String>,
}

impl AllowedMentions {
    pub fn sanitized() -> AllowedMentions {
        AllowedMentions {
            parse: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    url: String,
    reqwest: reqwest::Client,
}

impl Client {
    pub fn open(url: impl Into<String>) -> Client {
        Client {
            url: url.into(),
            reqwest: reqwest::Client::new(),
        }
    }

    pub async fn post(&self, payload: &Payload) -> reqwest::Result<()> {
        self.reqwest.post(&self.url)
            .json(payload)
            .send().await?;
        Ok(())
    }
}
