use serde_json::{Value, json};

use super::{App, Person};

pub struct Roles {
    pub owner: Person,
    pub admin: Person,
    pub member: Person,
    pub outsider: Person,
    pub space: String,
    pub other_space: String,
}

pub async fn roles(app: &App) -> Roles {
    let (owner, admin, member, outsider) = (
        app.person("owner").await,
        app.person("admin").await,
        app.person("member").await,
        app.person("outsider").await,
    );
    let space = app.space(&[(&owner, "owner"), (&admin, "admin"), (&member, "member")]).await;
    let other_space = app.space(&[(&outsider, "owner")]).await;
    Roles { owner, admin, member, outsider, space, other_space }
}

pub async fn page(app: &App, who: &Person, space: Option<&str>, markdown: &str) -> Value {
    let mut body = json!({"title": "Page", "markdown": markdown});
    if let Some(space) = space {
        body["space_id"] = json!(space);
    }
    let answer = app.call("POST", "/api/documents", Some(who), Some(body)).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    answer.json()
}

pub fn id(doc: &Value) -> String {
    doc["id"].as_str().unwrap().to_owned()
}
