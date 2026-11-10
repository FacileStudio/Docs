use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::command::{Command, Member, Out};
use crate::state::AppState;

/// The largest frame a member may send.
pub const MAX_FRAME: usize = 2 << 20;
/// How many frames wait for a slow member before it is closed with 1013.
pub const OUTBOX: usize = 512;

/// The longest close reason a WebSocket close frame carries, in bytes (RFC 6455 section 5.5).
pub const MAX_REASON: usize = 123;

/// A close frame whose reason is cut to [`MAX_REASON`] bytes on a character boundary: a longer
/// one fails to send and the client sees 1006 instead of the code.
pub fn close(code: u16, mut reason: String) -> Message {
    let cut = (0..=MAX_REASON.min(reason.len())).rev().find(|&at| reason.is_char_boundary(at)).unwrap_or(0);
    reason.truncate(cut);
    Message::Close(Some(CloseFrame { code, reason: reason.into() }))
}

/// Pumps one socket: joins the room, forwards frames both ways, leaves on close.
pub async fn serve(socket: WebSocket, state: AppState, document: Uuid, user_id: i64, editor: bool) {
    let (mut sink, mut stream) = socket.split();
    let (outbox, mut inbox) = mpsc::channel(OUTBOX);
    let conn = state.hub.next_id();
    let member = Member { conn, user_id, editor, outbox };
    if let Err(error) = state.hub.ask(&state, document, |reply| Command::Join(Member { ..member.clone() }, reply)).await
    {
        _ = sink.send(close(1011, error.message)).await;
        return;
    }
    loop {
        tokio::select! {
            incoming = stream.next() => match incoming {
                Some(Ok(Message::Binary(frame))) if frame.len() > MAX_FRAME => {
                    _ = sink.send(close(1009, "frame over 2 MiB".to_owned())).await;
                    break;
                }
                Some(Ok(Message::Binary(frame))) => state.hub.tell(document, Command::Message(conn, frame.to_vec())).await,
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => {}
            },
            out = inbox.recv() => match out {
                Some(Out::Binary(frame)) => if sink.send(Message::Binary(frame.into())).await.is_err() { break },
                Some(Out::Close(code, reason)) => { _ = sink.send(close(code, reason)).await; break }
                None => break,
            },
        }
    }
    state.hub.tell(document, Command::Leave(conn)).await;
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_long_close_reason_is_cut_on_a_character_boundary() {
        let axum::extract::ws::Message::Close(Some(frame)) = super::close(1008, "é".repeat(100)) else { panic!() };
        assert_eq!((frame.code, frame.reason.len()), (1008, 122));
    }
}
