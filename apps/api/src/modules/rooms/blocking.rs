//! The room's Yjs work on a blocking thread: the core is lent out and always given back.

use yrs::Doc;
use yrs::sync::Awareness;

use super::clients::Clients;
use super::sync::{Core, Effect, handle};

impl Core {
    /// A placeholder while the real core is lent to a blocking task.
    pub fn empty() -> Self {
        Core { doc: Doc::new(), awareness: Awareness::new(Doc::new()) }
    }
}

/// [`handle`] over owned halves, for a blocking task: always gives the core back, and turns a
/// panic into a 1011 refusal instead of losing the document.
pub fn handle_owned(mut core: Core, frame: &[u8], editor: bool, mut clients: Clients) -> (Core, Clients, Vec<Effect>) {
    let run = std::panic::AssertUnwindSafe(|| handle(&mut core, frame, editor, &mut clients));
    let effects =
        std::panic::catch_unwind(run).unwrap_or_else(|_| vec![Effect::Refuse(1011, "update failed".to_owned())]);
    (core, clients, effects)
}
