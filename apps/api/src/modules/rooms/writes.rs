use encre::ops::{self, Op};
use yrs::sync::{Message, SyncMessage};
use yrs::{ReadTxn, StateVector, Transact};

use super::command::Out;
use super::load::from_state;
use super::room::Room;
use super::sync::{self, encode};
use crate::modules::documents::content::Prepared;
use crate::modules::documents::ops::{MAX_OPS, refusal};
use crate::modules::documents::types::OpsAnswer;

type Edit = Box<dyn FnOnce(&mut yrs::TransactionMut, &yrs::XmlFragmentRef) -> tronc::Result<()> + Send>;

fn transact(doc: &yrs::Doc, edit: Edit) -> tronc::Result<Vec<u8>> {
    let fragment = doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
    let update = {
        let mut txn = doc.transact_mut();
        edit(&mut txn, &fragment).map(|()| txn.encode_update_v1())
    }?;
    sync::check(doc).map(|()| update).map_err(|r| tronc::Error::invalid(r.1))
}

impl Room {
    fn writable(&self) -> tronc::Result<()> {
        match self.trashed {
            true => Err(tronc::Error::failed("the document is in the trash")),
            false => Ok(()),
        }
    }

    async fn edit(&mut self, author: i64, force: bool, edit: Edit) -> tronc::Result<()> {
        self.writable()?;
        let before = self.core.doc.transact().encode_state_as_update_v1(&StateVector::default());
        let doc = self.core.doc.clone();
        let checked = tokio::task::spawn_blocking(move || transact(&doc, edit))
            .await
            .unwrap_or_else(|error| Err(tronc::Error::internal("the room edit did not finish", error)));
        let update = match checked {
            Ok(update) => update,
            Err(error) => {
                self.core.doc = from_state(&before).map_err(|e| tronc::Error::internal("room revert failed", e))?;
                return Err(error);
            }
        };
        self.broadcast(None, &encode(Message::Sync(SyncMessage::Update(update))));
        self.touch(Some(author));
        self.save(force).await;
        match self.dirty {
            None => Ok(()),
            Some(_) => Err(tronc::Error::unavailable("the edit is applied but not saved yet; it will be")),
        }
    }

    /// Agent operations as one Yjs transaction, persisted before the answer.
    pub(super) async fn ops(&mut self, list: Vec<Op>, author: i64) -> tronc::Result<OpsAnswer> {
        if list.len() > MAX_OPS {
            return Err(tronc::Error::invalid(format!("at most {MAX_OPS} operations per request")));
        }
        let version = self.version as u64;
        let reads_only = list.iter().all(|op| matches!(op, Op::Read | Op::ReadBlocks { .. }));
        let apply = move |txn: &mut yrs::TransactionMut, fragment: &yrs::XmlFragmentRef| {
            list.iter().try_for_each(|op| ops::apply_yjs(txn, fragment, version, op).map(|_| ()).map_err(refusal))
        };
        if !reads_only {
            self.edit(author, false, Box::new(apply)).await?;
        }
        let (prepared, _) = super::persist::snapshot(&self.core.doc)?;
        Ok(OpsAnswer { version: self.version, markdown: encre::markdown::serialize(&prepared.node) })
    }

    /// Replaces the whole document (a revision restore) and writes a revision.
    pub(super) async fn replace(&mut self, prepared: &Prepared, author: i64) -> tronc::Result<i64> {
        let node = prepared.node.clone();
        let write = move |txn: &mut yrs::TransactionMut, fragment: &yrs::XmlFragmentRef| {
            encre::yjs::write(txn, fragment, &node).map_err(|e| tronc::Error::invalid(e.to_string()))
        };
        self.edit(author, true, Box::new(write)).await?;
        Ok(self.version)
    }

    /// Persists, then closes every member with 1001.
    pub(super) async fn stop(&mut self) {
        self.save(true).await;
        let conns: Vec<u64> = self.conns.keys().copied().collect();
        for conn in conns {
            self.drop_conn(conn, Some(Out::Close(1001, "server restarting".to_owned())));
        }
    }
}
