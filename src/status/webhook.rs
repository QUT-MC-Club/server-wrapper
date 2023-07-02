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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<Embed>,
}

impl Payload {
    pub fn new_sanitized(content: String) -> Payload {
        Payload {
            content,
            username: None,
            avatar_url: None,
            embeds: Vec::new(),
            allowed_mentions: Some(AllowedMentions::sanitized()),
        }
    }
}

impl<T: Into<String>> From<T> for Payload {
    fn from(string: T) -> Self {
        Payload::new_sanitized(string.into())
    }
}

#[derive(Serialize)]
pub struct AllowedMentions {
    pub parse: Vec<String>,
}

impl AllowedMentions {
    pub fn sanitized() -> AllowedMentions {
        AllowedMentions { parse: Vec::new() }
    }
}

#[derive(Serialize)]
pub struct Embed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub ty: EmbedType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
}

#[derive(Serialize)]
pub enum EmbedType {
    #[serde(rename = "rich")]
    Rich,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "gifv")]
    Gifv,
    #[serde(rename = "article")]
    Article,
    #[serde(rename = "link")]
    Link,
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
        self.reqwest.post(&self.url).json(payload).send().await?;
        Ok(())
    }
}
