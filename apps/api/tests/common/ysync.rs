use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as Frame;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use yrs::sync::{Message, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, Transact, Update};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Peer {
    pub doc: Doc,
    socket: Option<Socket>,
    pub closed: Option<u16>,
}

impl Peer {
    pub fn new() -> Peer {
        let doc = encre::yjs::new_doc();
        doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
        Peer { doc, socket: None, closed: None }
    }

    pub async fn connect(&mut self, url: &str, token: &str) {
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
        let (socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
        self.socket = Some(socket);
        self.closed = None;
        let sv = self.doc.transact().state_vector();
        self.send(Message::Sync(SyncMessage::SyncStep1(sv))).await;
        self.pump(Duration::from_millis(500)).await;
    }

    pub async fn send_raw(&mut self, bytes: Vec<u8>) {
        if let Some(socket) = self.socket.as_mut() {
            _ = socket.send(Frame::Binary(bytes.into())).await;
        }
    }

    pub fn disconnect(&mut self) {
        self.socket = None;
    }

    pub async fn send(&mut self, message: Message) {
        if let Some(socket) = self.socket.as_mut() {
            _ = socket.send(Frame::Binary(message.encode_v1().into())).await;
        }
    }

    fn on(&mut self, message: Message) -> Option<Message> {
        match message {
            Message::Sync(SyncMessage::SyncStep1(sv)) => {
                Some(Message::Sync(SyncMessage::SyncStep2(self.doc.transact().encode_state_as_update_v1(&sv))))
            }
            Message::Sync(SyncMessage::SyncStep2(u) | SyncMessage::Update(u)) => {
                self.doc.transact_mut().apply_update(Update::decode_v1(&u).unwrap()).unwrap();
                None
            }
            _ => None,
        }
    }

    pub async fn pump(&mut self, quiet: Duration) {
        loop {
            let Some(socket) = self.socket.as_mut() else { return };
            let frame = match tokio::time::timeout(quiet, socket.next()).await {
                Err(_) => return,
                Ok(frame) => frame,
            };
            match frame {
                Some(Ok(Frame::Binary(bytes))) => {
                    let mut decoder = yrs::updates::decoder::DecoderV1::from(bytes.as_ref());
                    let replies: Vec<Message> = yrs::sync::protocol::MessageReader::new(&mut decoder)
                        .filter_map(|m| m.ok())
                        .filter_map(|m| self.on(m))
                        .collect();
                    for reply in replies {
                        self.send(reply).await;
                    }
                }
                Some(Ok(Frame::Close(frame))) => {
                    self.closed = Some(frame.map_or(1005, |f| u16::from(f.code)));
                    self.socket = None;
                    return;
                }
                Some(Ok(_)) => {}
                Some(Err(_)) | None => {
                    self.closed.get_or_insert(u16::from(CloseCode::Abnormal));
                    self.socket = None;
                    return;
                }
            }
        }
    }

    pub async fn edit(&mut self, change: impl FnOnce(&mut yrs::TransactionMut, &yrs::XmlFragmentRef)) {
        let fragment = self.doc.get_or_insert_xml_fragment(encre::yjs::FRAGMENT);
        let update = {
            let mut txn = self.doc.transact_mut();
            change(&mut txn, &fragment);
            txn.encode_update_v1()
        };
        self.send(Message::Sync(SyncMessage::Update(update))).await;
    }
}
