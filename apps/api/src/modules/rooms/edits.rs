//! Server-side edits of a page: agent operations and revision restores, run inside the room.

use encre::ops::{self, Op};
use yrs::{Transact, TransactionMut, XmlFragmentRef};

use super::load::from_state;
use super::persist::snapshot;
use super::space::{Page, check, state};
use crate::modules::documents::content::Prepared;
use crate::modules::documents::ops::refusal;
use crate::modules::documents::types::OpsAnswer;

/// Runs `edit` on the page in one transaction and checks the result. A refused edit leaves the
/// page exactly as it was; a page in the trash refuses every edit.
fn transact(
    page: &mut Page,
    edit: impl FnOnce(&mut TransactionMut, &XmlFragmentRef) -> tronc::Result<()>,
) -> tronc::Result<Option<Vec<u8>>> {
    if page.trashed {
        return Err(tronc::Error::failed("the document is in the trash"));
    }
    let before = state(&page.doc);
    let fragment = page.doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    let update = {
        let mut txn = page.doc.transact_mut();
        edit(&mut txn, &fragment).map(|()| txn.encode_update_v1())
    };
    let checked = update.and_then(|update| check(&page.doc).map(|()| update).map_err(|r| tronc::Error::invalid(r.1)));
    if checked.is_err() {
        page.doc = from_state(&before).map_err(|e| tronc::Error::internal("room revert failed", e))?;
    }
    checked.map(Some)
}

/// Agent operations as one transaction at the page's current version. Reads change nothing.
pub fn apply(list: Vec<Op>) -> impl FnOnce(&mut Page) -> tronc::Result<Option<Vec<u8>>> + Send + 'static {
    move |page| {
        if list.iter().all(|op| matches!(op, Op::Read | Op::ReadBlocks { .. })) {
            return Ok(None);
        }
        let version = page.version as u64;
        transact(page, |txn, fragment| {
            list.iter().try_for_each(|op| ops::apply_yjs(txn, fragment, version, op).map(|_| ()).map_err(refusal))
        })
    }
}

/// What agent operations answer: the page's markdown and version once saved.
pub fn answered(page: &Page, saved: bool) -> tronc::Result<OpsAnswer> {
    let (prepared, _) = snapshot(&page.doc)?;
    Ok(OpsAnswer { version: page.version, markdown: encre::markdown::serialize(&prepared.node), saved })
}

/// The whole page replaced by `prepared` (a revision restore).
pub fn replace(prepared: Box<Prepared>) -> impl FnOnce(&mut Page) -> tronc::Result<Option<Vec<u8>>> + Send + 'static {
    move |page| {
        transact(page, |txn, fragment| {
            encre::yjs::write(txn, fragment, &prepared.node).map_err(|e| tronc::Error::invalid(e.to_string()))
        })
    }
}
