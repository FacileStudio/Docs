use schemars::JsonSchema;
use serde::Serialize;
use tronc::mcp::{ToolError, cursor, encoded_size, fit_count, offset};

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

/// The window of `items` that `at` asks for, cut further so it stays within the result budget.
/// A cursor is tronc's offset cursor; a window is never empty while items remain.
pub fn window<T: Serialize>(items: Vec<T>, at: Window) -> Result<Listed<T>, ToolError> {
    let limit = at.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(ToolError(format!("limit is 1 to {MAX_LIMIT}, not {limit}")));
    }
    let total = items.len();
    let start = offset(at.cursor.as_deref().unwrap_or(""))?.min(total);
    let mut pages: Vec<T> = items.into_iter().skip(start).take(limit).collect();
    let fit = fit_count(pages.len(), |n| encoded_size(&pages[..n])).max(1);
    pages.truncate(fit);
    let next = cursor(start + pages.len(), total);
    Ok(Listed { pages, next_cursor: (!next.is_empty()).then_some(next), total })
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
        assert_eq!((first.pages, first.total), (vec![0, 1], 5));
        let last = window((0..5).collect(), at(Some(&cursor(4, 5)), Some(2))).unwrap();
        assert_eq!((last.pages, last.next_cursor), (vec![4], None));
        assert_eq!(first.next_cursor, Some(cursor(2, 5)));
        assert!(window::<i32>(vec![], at(Some("x"), None)).unwrap_err().0.contains("cursor is not valid"));
        assert!(window::<i32>(vec![], at(None, Some(0))).unwrap_err().0.contains("limit"));
        assert!(window::<i32>(vec![], at(None, Some(201))).is_err());
    }

    #[test]
    fn a_window_stops_at_the_result_budget() {
        let big = "x".repeat(10_000);
        let listed = window(vec![big; 20], at(None, Some(20))).unwrap();
        assert!((1..20).contains(&listed.pages.len()), "{}", listed.pages.len());
        assert!(encoded_size(&listed) <= tronc::mcp::RESULT_BUDGET + 100);
        let rest = window(vec!["y"; 20], at(listed.next_cursor.as_deref(), Some(20))).unwrap();
        assert_eq!(rest.pages.len(), 20 - listed.pages.len());
    }
}
