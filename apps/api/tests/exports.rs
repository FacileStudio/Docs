mod common;

use common::fixture;
use common::upload::upload;
use serde_json::json;

const EVERY: &str = r#"# Heading one

## Heading two

A paragraph with **bold**, *italic*, ~~struck~~, `code` and a [link](https://example.test).

- bullet item
- [x] task done

1. ordered item

> quoted text

```rust
let fenced = 1;
```

---

| Column | Other |
| --- | --- |
| cell text | more |

````callout tone="warning" title="Callout title"
Callout body
````

```stat
{"label": "Revenue label", "value": "12400", "delta": "+4%"}
```

```chart
{"kind": "bar", "title": "Chart title", "labels": ["Mon", "Tue"], "series": [{"name": "web", "data": [3, 5]}]}
```
"#;

const STRINGS: &[&str] = &[
    "Heading one",
    "Heading two",
    "bold",
    "italic",
    "struck",
    "code",
    "link",
    "bullet item",
    "task done",
    "ordered item",
    "quoted text",
    "fenced",
    "Column",
    "cell text",
    "Callout title",
    "Callout body",
    "Revenue label",
    "12400",
    "Chart title",
    "Mon",
    "web",
    "picture alt",
];

#[tokio::test]
async fn a_page_with_every_component_exports_to_a_pdf_holding_every_string() {
    let Some(pool) = common::pool("docs_pdf").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, EVERY).await);
    let png = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/testdata/dot.png")).unwrap();
    let url = upload(&app, &roles.member, &id, "dot.png", &png).await.json()["url"].as_str().unwrap().to_owned();
    let ops = json!({"ops": [{"op": "append", "markdown": format!("![picture alt]({url})")}]});
    assert_eq!(app.call("POST", &format!("/api/documents/{id}/ops"), Some(&roles.member), Some(ops)).await.status, 200);
    let pdf = app.call("GET", &format!("/api/documents/{id}/export.pdf"), Some(&roles.member), None).await;
    assert_eq!(pdf.status, 200, "{}", pdf.text());
    assert_eq!(pdf.headers["content-type"], "application/pdf");
    let text = pdf_extract::extract_text_from_mem(&pdf.bytes).unwrap();
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let missing: Vec<&&str> = STRINGS.iter().filter(|s| !flat.contains(**s) && **s != "picture alt").collect();
    assert!(missing.is_empty(), "missing {missing:?} in {flat}");
    assert!(pdf.bytes.windows(6).any(|w| w == b"/Image"), "the attachment image is not embedded");
    assert_eq!(
        app.call("GET", &format!("/api/documents/{id}/export.pdf"), Some(&roles.outsider), None).await.status,
        404
    );
}

#[tokio::test]
async fn markdown_exports_are_facile_and_commonmark() {
    let Some(pool) = common::pool("docs_md_export").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::page(&app, &roles.member, None, "Line one\n\n\\\n\nNext").await);
    let md = app.call("GET", &format!("/api/documents/{id}/export.md"), Some(&roles.member), None).await;
    assert_eq!(md.headers["content-type"], "text/markdown; charset=utf-8");
    assert_eq!(md.headers["content-disposition"], "attachment; filename=\"Page.md\"");
    let cm = app.call("GET", &format!("/api/documents/{id}/export.commonmark"), Some(&roles.member), None).await;
    assert_eq!(md.text(), "Line one\n\n\\\n\nNext");
    assert_eq!(cm.text(), "Line one\n\nNext");
}

#[tokio::test]
async fn an_openui_program_becomes_a_page() {
    let Some(pool) = common::pool("docs_openui").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let source = "root = Card([s], \"Weekly\")\ns = Stat(\"Signups\", 42, \"+3\")";
    let body = json!({"source": source, "title": "From Bulle", "space_id": roles.space});
    let created = app.call("POST", "/api/documents/import-openui", Some(&roles.member), Some(body.clone())).await;
    assert_eq!(created.status, 201, "{}", created.text());
    assert_eq!(created.json()["content"]["content"][0]["type"], json!("heading"));
    assert_eq!(app.call("POST", "/api/documents/import-openui", Some(&roles.outsider), Some(body)).await.status, 404);
    let bad = json!({"source": "not openui ((", "title": "x"});
    assert_eq!(app.call("POST", "/api/documents/import-openui", Some(&roles.member), Some(bad)).await.status, 400);
}

#[tokio::test]
async fn every_mounted_route_is_in_the_api_reference() {
    let Some(pool) = common::pool("docs_apiref").await else { return };
    let app = common::app(pool).await;
    let missing = tronc::apiref::undocumented(&app.router, &api::apiref::document(), &api::apiref::config(), &[]);
    assert!(missing.is_empty(), "routes missing from the API document: {missing:?}");
    assert_eq!(app.call("GET", "/docs", None, None).await.status, 200);
    assert_eq!(app.call("GET", "/docs/openapi.json", None, None).await.json()["info"]["title"], json!("Docs API"));
}
