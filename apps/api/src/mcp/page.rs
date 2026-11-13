use schemars::JsonSchema;
use serde::Serialize;
use tronc::mcp::ToolError;

/// The window a list tool answers when the call names no `limit`.
pub const DEFAULT_LIMIT: usize = 50;

/// The largest window a list tool answers.
pub const MAX_LIMIT: usize = 200;

/// Where a list call starts and how many items it wants.
pub struct Window {
    /// The `next_cursor` of the previous call.
    pub cursor: Option<String>,
    /// How many items, 1 to [`MAX_LIMIT`].
    pub limit: Option<usize>,
}

/// One window of a list.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Listed<T: Serialize> {
    pub pages: Vec<T>,
    /// Pass it as `cursor` for the next window; absent on the last one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// How many items the whole list holds.
    pub total: usize,
}

/// The window of `items` that `at` asks for. A cursor is an offset the previous window wrote.
pub fn window<T: Serialize>(items: Vec<T>, at: Window) -> Result<Listed<T>, ToolError> {
    let limit = at.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(ToolError(format!("limit is 1 to {MAX_LIMIT}, not {limit}")));
    }
    let start = match at.cursor.as_deref() {
        None | Some("") => 0,
        Some(raw) => raw.parse::<usize>().map_err(|_| {
            ToolError(format!("cursor {raw:?} is not one this tool wrote; pass next_cursor as it came, or omit it"))
        })?,
    };
    let total = items.len();
    let pages: Vec<T> = items.into_iter().skip(start).take(limit).collect();
    let end = start.saturating_add(pages.len());
    let next_cursor = (end < total).then(|| end.to_string());
    Ok(Listed { pages, next_cursor, total })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(cursor: Option<&str>, limit: Option<usize>) -> Window {
        Window { cursor: cursor.map(str::to_owned), limit }
    }

    #[test]
    fn windows_walk_the_list_and_refuse_bad_cursors_and_limits() {
        let first = window((0..5).collect(), at(None, Some(2))).unwrap();
        assert_eq!((first.pages, first.next_cursor.as_deref(), first.total), (vec![0, 1], Some("2"), 5));
        let last = window((0..5).collect(), at(Some("4"), Some(2))).unwrap();
        assert_eq!((last.pages, last.next_cursor), (vec![4], None));
        assert!(window::<i32>(vec![], at(Some("x"), None)).unwrap_err().0.contains("next_cursor"));
        assert!(window::<i32>(vec![], at(None, Some(0))).unwrap_err().0.contains("limit"));
        assert!(window::<i32>(vec![], at(None, Some(201))).is_err());
    }
}
