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

fn close(code: u16, reason: String) -> Message {
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
