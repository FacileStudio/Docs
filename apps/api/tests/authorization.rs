mod common;

use common::fixture::{self, Roles};
use common::{App, Person};
use serde_json::{Value, json};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Need {
    Viewer,
    Editor,
    Manager,
}

#[derive(Clone, Copy)]
enum Prep {
    None,
    Trashed,
    Revision,
}

struct Case {
    method: &'static str,
    path: &'static str,
    body: Option<fn() -> Value>,
    need: Need,
    prep: Prep,
    ok: u16,
}

type Row = (&'static str, &'static str, Option<fn() -> Value>, Need, Prep, u16);

fn case((method, path, body, need, prep, ok): Row) -> Case {
    Case { method, path, body, need, prep, ok }
}

fn content_cases() -> Vec<Case> {
    vec![
        case(("GET", "/api/documents/{id}", None, Need::Viewer, Prep::None, 200)),
        case(("GET", "/api/documents/{id}?format=markdown", None, Need::Viewer, Prep::None, 200)),
        case(("PATCH", "/api/documents/{id}", Some(|| json!({"title": "Renamed"})), Need::Editor, Prep::None, 200)),
        case((
            "POST",
            "/api/documents/{id}/ops",
            Some(|| json!({"ops": [{"op": "append", "markdown": "more"}]})),
            Need::Editor,
            Prep::None,
            200,
        )),
        case(("POST", "/api/documents/{id}/move", Some(|| json!({})), Need::Manager, Prep::None, 200)),
        case(("DELETE", "/api/documents/{id}", None, Need::Manager, Prep::None, 204)),
        case(("POST", "/api/documents/{id}/restore", None, Need::Manager, Prep::Trashed, 200)),
    ]
}

fn cases() -> Vec<Case> {
    let mut all = content_cases();
    all.extend([
        case(("DELETE", "/api/documents/{id}/permanent", None, Need::Manager, Prep::Trashed, 204)),
        case(("GET", "/api/documents/{id}/revisions", None, Need::Viewer, Prep::Revision, 200)),
        case(("POST", "/api/documents/{id}/revisions/{rev}/restore", None, Need::Editor, Prep::Revision, 200)),
        case((
            "POST",
            "/api/documents",
            Some(|| json!({"title": "Child", "parent_id": "{id}"})),
            Need::Editor,
            Prep::None,
            201,
        )),
        case(("POST", "/api/documents/{id}/share", Some(|| json!({"children": true})), Need::Manager, Prep::None, 201)),
        case(("GET", "/api/documents/{id}/share", None, Need::Manager, Prep::None, 200)),
        case(("GET", "/api/documents/{id}/export.md", None, Need::Viewer, Prep::None, 200)),
        case(("GET", "/api/documents/{id}/export.commonmark", None, Need::Viewer, Prep::None, 200)),
    ]);
    all
}

async fn prepared(app: &App, roles: &Roles, prep: Prep) -> (String, String) {
    let doc = fixture::id(&fixture::page(app, &roles.owner, Some(&roles.space), "# Title\n\nBody").await);
    let mut rev = String::new();
    match prep {
        Prep::None => {}
        Prep::Trashed => {
            assert_eq!(app.call("DELETE", &format!("/api/documents/{doc}"), Some(&roles.owner), None).await.status, 204)
        }
        Prep::Revision => {
            let ops = json!({"ops": [{"op": "append", "markdown": "v2"}]});
            assert_eq!(
                app.call("POST", &format!("/api/documents/{doc}/ops"), Some(&roles.owner), Some(ops)).await.status,
                200
            );
            let list =
                app.call("GET", &format!("/api/documents/{doc}/revisions"), Some(&roles.owner), None).await.json();
            rev = list[0]["id"].as_str().unwrap().to_owned();
        }
    }
    (doc, rev)
}

async fn attempt(app: &App, roles: &Roles, case: &Case, who: Option<&Person>) -> u16 {
    let (doc, rev) = prepared(app, roles, case.prep).await;
    let path = case.path.replace("{id}", &doc).replace("{rev}", &rev);
    let body = case.body.map(|body| serde_json::from_str(&body().to_string().replace("{id}", &doc)).unwrap());
    let answer = app.call(case.method, &path, who, body).await;
    answer.status.as_u16()
}

fn expected(case: &Case, have: Option<Need>) -> u16 {
    match have {
        None => 404,
        Some(have) if have < case.need => 403,
        Some(_) => case.ok,
    }
}

#[tokio::test]
async fn every_route_answers_each_role_as_its_access_allows() {
    let Some(pool) = common::pool("docs_authz").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let people = [
        (&roles.owner, Some(Need::Manager)),
        (&roles.admin, Some(Need::Manager)),
        (&roles.member, Some(Need::Editor)),
        (&roles.outsider, None),
    ];
    for case in cases() {
        assert_eq!(attempt(&app, &roles, &case, None).await, 401, "anonymous {} {}", case.method, case.path);
        for (person, have) in &people {
            let got = attempt(&app, &roles, &case, Some(person)).await;
            assert_eq!(got, expected(&case, *have), "{} {} as {:?}", case.method, case.path, person.subject);
        }
    }
}
