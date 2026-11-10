use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

/// Page size of every list call.
pub const PAGE: usize = 100;

type ClientError = Box<dyn std::error::Error + Send + Sync>;

/// An Outline API client: every method is a `POST /api/<method>` with a JSON body.
pub struct Client {
    pub base: String,
    pub token: String,
    pub http: reqwest::Client,
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

/// One collection, as `collections.list` answers it.
#[derive(Debug, Clone, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A node of `collections.documents`: the ordered tree of a collection.
#[derive(Debug, Clone, Deserialize)]
pub struct NavNode {
    pub id: String,
    #[serde(default)]
    pub children: Vec<NavNode>,
}

/// The author of a document.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Author {
    #[serde(default)]
    pub email: Option<String>,
}

/// One document, as `documents.list` answers it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub url_id: String,
    pub title: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub parent_document_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub created_by: Option<Author>,
}

impl Client {
    async fn call<T: DeserializeOwned>(&self, method: &str, body: Value) -> Result<T, ClientError> {
        let url = format!("{}/api/{method}", self.base.trim_end_matches('/'));
        let response = self.http.post(url).bearer_auth(&self.token).json(&body).send().await?.error_for_status()?;
        Ok(response.json::<Envelope<T>>().await?.data)
    }

    async fn all<T: DeserializeOwned>(&self, method: &str, mut body: Value) -> Result<Vec<T>, ClientError> {
        let mut out = Vec::new();
        loop {
            body["offset"] = json!(out.len());
            body["limit"] = json!(PAGE);
            let page: Vec<T> = self.call(method, body.clone()).await?;
            let done = page.len() < PAGE;
            out.extend(page);
            if done {
                return Ok(out);
            }
        }
    }

    /// `collections.list`, every page.
    pub async fn collections(&self) -> Result<Vec<Collection>, ClientError> {
        self.all("collections.list", json!({})).await
    }

    /// `collections.documents`: the ordered tree of one collection.
    pub async fn tree(&self, collection: &str) -> Result<Vec<NavNode>, ClientError> {
        self.call("collections.documents", json!({"id": collection})).await
    }

    /// `documents.list` for one collection, every page.
    pub async fn documents(&self, collection: &str) -> Result<Vec<Document>, ClientError> {
        self.all("documents.list", json!({"collectionId": collection})).await
    }

    /// `documents.export`: the markdown of one document.
    pub async fn export(&self, id: &str) -> Result<String, ClientError> {
        self.call("documents.export", json!({"id": id})).await
    }

    /// The bytes behind an attachment URL of this instance (`/api/attachments.redirect?id=…`).
    pub async fn attachment(&self, path: &str) -> Result<Vec<u8>, ClientError> {
        let url = format!("{}{path}", self.base.trim_end_matches('/'));
        let response = self.http.get(url).bearer_auth(&self.token).send().await?.error_for_status()?;
        Ok(response.bytes().await?.to_vec())
    }
}
