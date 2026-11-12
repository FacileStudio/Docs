use std::collections::HashSet;

use yrs::sync::{Awareness, Message, SyncMessage};
use yrs::updates::decoder::{Decode, DecoderV1};
use yrs::updates::encoder::Encode;
use yrs::{ClientID, Doc, ReadTxn, StateVector, Transact, Update};

use super::clients::Clients;
use super::load::from_state;
use crate::modules::documents::content;

/// A room's two Yjs halves: the document, and the presence of its members.
pub struct Core {
    /// The `default` fragment.
    pub doc: Doc,
    /// Cursors and names, relayed, never stored.
    pub awareness: Awareness,
}

/// What a frame asks the room to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    /// Send back to the sender.
    Reply(Vec<u8>),
    /// Send to every other member.
    Broadcast(Vec<u8>),
    /// The document changed: persist soon.
    Changed,
    /// Close the sender's socket with this code.
    Refuse(u16, String),
}

/// The update was refused and undone.
#[derive(Debug, PartialEq, Eq)]
pub struct Refusal(pub u16, pub String);

/// Encodes one y-sync message.
pub fn encode(message: Message) -> Vec<u8> {
    message.encode_v1()
}

/// What a joining member receives: the server's state vector, then everyone's presence.
pub fn opening(core: &Core) -> Vec<Vec<u8>> {
    let sv = core.doc.transact().state_vector();
    let mut out = vec![encode(Message::Sync(SyncMessage::SyncStep1(sv)))];
    if let Some(update) = core.awareness.update().ok().filter(|u| !u.clients.is_empty()) {
        out.push(encode(Message::Awareness(update)));
    }
    out
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

/// Applies `update` and keeps it only if [`check`] passes; otherwise the document goes back to
/// the state before it, exactly.
pub fn apply_checked(doc: &mut Doc, update: &[u8]) -> Result<(), Refusal> {
    let before = doc.transact().encode_state_as_update_v1(&StateVector::default());
    let decoded = Update::decode_v1(update).map_err(|e| Refusal(1008, format!("undecodable update: {e}")))?;
    let applied = doc.transact_mut().apply_update(decoded).map_err(|e| Refusal(1008, format!("update refused: {e}")));
    let refusal = match applied.and_then(|()| check(doc)) {
        Ok(()) => return Ok(()),
        Err(refusal) => refusal,
    };
    *doc = from_state(&before).map_err(|e| Refusal(1011, format!("revert failed: {e}")))?;
    Err(refusal)
}

fn on_update(core: &mut Core, update: Vec<u8>, editor: bool) -> Vec<Effect> {
    if !editor {
        return Vec::new();
    }
    match apply_checked(&mut core.doc, &update) {
        Ok(()) => vec![Effect::Broadcast(encode(Message::Sync(SyncMessage::Update(update)))), Effect::Changed],
        Err(Refusal(code, reason)) => vec![Effect::Refuse(code, reason)],
    }
}

fn on_message(core: &mut Core, message: Message, editor: bool, clients: &mut Clients) -> Vec<Effect> {
    match message {
        Message::Sync(SyncMessage::SyncStep1(sv)) => {
            let update = core.doc.transact().encode_state_as_update_v1(&sv);
            vec![Effect::Reply(encode(Message::Sync(SyncMessage::SyncStep2(update))))]
        }
        Message::Sync(SyncMessage::SyncStep2(update) | SyncMessage::Update(update)) => on_update(core, update, editor),
        Message::Awareness(mut update) => {
            update.clients.retain(|id, _| clients.claim(*id));
            if update.clients.is_empty() {
                return Vec::new();
            }
            match core.awareness.apply_update(update.clone()) {
                Ok(()) => vec![Effect::Broadcast(encode(Message::Awareness(update)))],
                Err(_) => Vec::new(),
            }
        }
        Message::AwarenessQuery => core
            .awareness
            .update()
            .map_or_else(|_| Vec::new(), |update| vec![Effect::Reply(encode(Message::Awareness(update)))]),
        Message::Auth(_) | Message::Custom(..) => Vec::new(),
    }
}

/// Handles one binary frame from a member. A viewer's document updates are dropped; its
/// presence is relayed, for the ids it owns only. Processing stops at the first refusal.
pub fn handle(core: &mut Core, frame: &[u8], editor: bool, clients: &mut Clients) -> Vec<Effect> {
    let mut decoder = DecoderV1::from(frame);
    let mut effects = Vec::new();
    for message in yrs::sync::protocol::MessageReader::new(&mut decoder) {
        let Ok(message) = message else {
            effects.push(Effect::Refuse(1008, "malformed y-sync frame".to_owned()));
            break;
        };
        effects.extend(on_message(core, message, editor, clients));
        if matches!(effects.last(), Some(Effect::Refuse(..))) {
            break;
        }
    }
    effects
}

/// The presence update that tells everyone `clients` left.
pub fn departure(awareness: &mut Awareness, clients: &HashSet<ClientID>) -> Option<Vec<u8>> {
    clients.iter().for_each(|client| awareness.remove_state(*client));
    let update = awareness.update_with_clients(clients.iter().copied()).ok()?;
    (!update.clients.is_empty()).then(|| encode(Message::Awareness(update)))
}
