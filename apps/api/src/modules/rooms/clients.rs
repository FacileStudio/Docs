//! Which presence ids a connection may speak for.

use std::collections::HashSet;

use yrs::ClientID;

/// The presence ids one connection may speak for: the ones it claimed first, never another
/// connection's.
#[derive(Debug, Default)]
pub struct Clients {
    /// Ids this connection claimed.
    pub own: HashSet<ClientID>,
    /// Ids the room's other connections claimed.
    pub taken: HashSet<ClientID>,
}

impl Clients {
    /// Claims `id` for this connection, unless another connection owns it.
    pub fn claim(&mut self, id: ClientID) -> bool {
        match self.taken.contains(&id) {
            true => false,
            false => {
                self.own.insert(id);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use yrs::sync::awareness::{AwarenessUpdate, AwarenessUpdateEntry};

    use std::collections::HashSet;

    use yrs::sync::{Awareness, Message};
    use yrs::{ClientID, Doc};

    use super::Clients;
    use crate::modules::rooms::sync::{Core, encode, handle};

    fn presence(id: u64, clock: u32, name: &str) -> Vec<u8> {
        let entry = AwarenessUpdateEntry { clock, json: format!("{{\"user\":{{\"name\":\"{name}\"}}}}").into() };
        encode(Message::Awareness(AwarenessUpdate { clients: HashMap::from([(ClientID::new(id), entry)]) }))
    }

    #[test]
    fn a_connection_cannot_speak_for_a_presence_id_another_one_owns() {
        let (seven, nine) = (ClientID::new(7), ClientID::new(9));
        let mut core = Core { doc: Doc::new(), awareness: Awareness::new(Doc::new()) };
        let mut alice = Clients::default();
        assert_eq!(handle(&mut core, &presence(7, 1, "Alice"), false, &mut alice).len(), 1);
        assert!(alice.own.contains(&seven));
        let mut mallory = Clients { own: HashSet::new(), taken: alice.own.clone() };
        assert!(handle(&mut core, &presence(7, 2, "Mallory"), true, &mut mallory).is_empty());
        assert!(mallory.own.is_empty());
        assert_eq!(core.awareness.meta(seven).map(|(clock, _)| clock), Some(1));
        assert_eq!(handle(&mut core, &presence(9, 1, "Mallory"), true, &mut mallory).len(), 1);
        assert!(mallory.own.contains(&nine));
    }
}
