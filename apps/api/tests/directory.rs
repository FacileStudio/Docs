mod common;

use api::auth::{AppData, SpaceStore, SpacesApply};
use common::fixture;
use porte::directory::{Member, PersonDeleted, Snapshot, Space};
use serde_json::json;

#[tokio::test]
async fn the_space_store_passes_porte_conformance() {
    let Some(pool) = common::pool("docs_conformance").await else { return };
    let result = porte::spaces::conformance::run(|| {
        let pool = pool.clone();
        async move {
            sqlx::query("TRUNCATE spaces CASCADE").execute(&pool).await.unwrap();
            SpaceStore::new(pool)
        }
    })
    .await;
    assert!(result.is_ok(), "{:#?}", result.unwrap_err());
}

fn snapshot(version: i64, spaces: Vec<Space>) -> Snapshot {
    Snapshot { version, spaces, ..Snapshot::default() }
}

fn space(id: &str, subjects: &[(&str, &str)]) -> Space {
    let members = subjects
        .iter()
        .map(|(subject, role)| Member { subject: subject.to_string(), role: role.to_string(), ..Member::default() })
        .collect();
    Space { id: id.into(), name: format!("Space {id}"), members, ..Space::default() }
}

#[tokio::test]
async fn a_snapshot_replaces_spaces_and_members_once_per_version() {
    let Some(pool) = common::pool("docs_directory").await else { return };
    let apply = SpacesApply::new(pool.clone());
    let base = time::OffsetDateTime::now_utc().unix_timestamp_nanos() as i64;
    assert!(
        apply.apply_snapshot(&snapshot(base, vec![space("a", &[("s1", "owner")]), space("b", &[])])).await.unwrap()
    );
    assert!(!apply.apply_snapshot(&snapshot(base, vec![])).await.unwrap());
    assert!(apply.apply_snapshot(&snapshot(base + 1, vec![space("a", &[("s2", "member")])])).await.unwrap());
    let spaces: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces ORDER BY id").fetch_all(&pool).await.unwrap();
    assert_eq!(spaces, ["a"]);
    let members: Vec<String> = sqlx::query_scalar("SELECT subject FROM space_members").fetch_all(&pool).await.unwrap();
    assert_eq!(members, ["s2"]);
}

#[tokio::test]
async fn erasing_a_person_removes_their_pages_memberships_and_account() {
    let Some(pool) = common::pool("docs_erase").await else { return };
    let app = common::app(pool.clone()).await;
    let roles = fixture::roles(&app).await;
    let personal = fixture::id(&fixture::page(&app, &roles.member, None, "mine").await);
    let shared = fixture::id(&fixture::page(&app, &roles.member, Some(&roles.space), "ours").await);
    AppData::new(pool.clone()).on_person_deleted(common::ISSUER, &roles.member.subject).await.unwrap();
    let ids = [uuid::Uuid::parse_str(&personal).unwrap(), uuid::Uuid::parse_str(&shared).unwrap()];
    let left: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE id = ANY($1)")
        .bind(&ids[..])
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, [ids[1]]);
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM porte_users WHERE id = $1")
        .bind(roles.member.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 0);
    let owner = app.call("GET", &format!("/api/documents/{shared}"), Some(&roles.owner), None).await;
    assert_eq!(owner.status, 200);
    assert_eq!(app.call("GET", "/api/spaces", Some(&roles.member), None).await.status, 401);
    assert_eq!(owner.json()["title"], json!("Page"));
}
