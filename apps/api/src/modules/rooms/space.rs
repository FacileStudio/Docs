//! A room's document: the page's Yjs document, checked as a stored body after every update.

use salon::{Refusal, Space};
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use super::load::from_state;
use crate::modules::documents::content;

/// The page of one room, with the stored version and whether it is in the trash.
#[derive(Default)]
pub struct Page {
    /// The `default` fragment.
    pub doc: Doc,
    /// The stored version, as of the last save.
    pub version: i64,
    /// A page in the trash makes everyone a viewer.
    pub trashed: bool,
}

/// Checks the document after an update: it must read and validate, and stay under 5 MiB.
pub fn check(doc: &Doc) -> Result<(), Refusal> {
    let fragment = doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    let txn = doc.transact();
    let node = encre::yjs::read(&txn, &fragment).map_err(|e| Refusal(1008, format!("unreadable document: {e}")))?;
    content::measure(node).map(|_| ()).map_err(|error| match error.code {
        tronc::Code::ResourceExhausted => Refusal(1009, error.message),
        _ => Refusal(1008, error.message),
    })
}

/// The whole state of `doc`, as one v1 update.
pub fn state(doc: &Doc) -> Vec<u8> {
    doc.transact().encode_state_as_update_v1(&StateVector::default())
}

impl Space for Page {
    fn doc(&self) -> &Doc {
        &self.doc
    }

    /// Applies `update` and keeps it only if [`check`] passes; otherwise the document goes back
    /// to the state before it, exactly.
    fn apply(&mut self, update: &[u8]) -> Result<bool, Refusal> {
        let before = state(&self.doc);
        let decoded = Update::decode_v1(update).map_err(|e| Refusal(1008, format!("undecodable update: {e}")))?;
        let applied =
            self.doc.transact_mut().apply_update(decoded).map_err(|e| Refusal(1008, format!("update refused: {e}")));
        let refusal = match applied.and_then(|()| check(&self.doc)) {
            Ok(()) => return Ok(true),
            Err(refusal) => refusal,
        };
        self.doc = from_state(&before).map_err(|e| Refusal(1011, format!("revert failed: {e}")))?;
        Err(refusal)
    }

    fn writable(&self) -> bool {
        !self.trashed
    }
}
