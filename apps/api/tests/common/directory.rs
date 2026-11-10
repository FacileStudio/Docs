use porte::directory::{Member, Snapshot, Space};

pub fn snapshot(version: i64, spaces: Vec<Space>) -> Snapshot {
    Snapshot { version, spaces, ..Snapshot::default() }
}

pub fn space(id: &str, subjects: &[(&str, &str)]) -> Space {
    let members = subjects
        .iter()
        .map(|(subject, role)| Member { subject: subject.to_string(), role: role.to_string(), ..Member::default() })
        .collect();
    Space { id: id.into(), name: format!("Space {id}"), members, ..Space::default() }
}

pub fn base() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp_nanos() as i64
}
