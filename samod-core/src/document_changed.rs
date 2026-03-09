use automerge::ChangeHash;

use crate::{ConnectionId, PeerId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeOrigin {
    Local,
    Remote {
        connection_id: ConnectionId,
        peer_id: PeerId,
    },
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentChanged {
    pub old_heads: Vec<ChangeHash>,
    pub new_heads: Vec<ChangeHash>,
    pub origin: ChangeOrigin,
}
