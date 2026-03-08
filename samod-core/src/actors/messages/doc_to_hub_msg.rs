use std::collections::HashMap;

use crate::{
    ConnectionId, DocumentId, actors::document::DocumentStatus, ephemera::EphemeralMessage,
    network::PeerDocState,
};

use super::SyncMessage;

/// Messages sent from document actors to Samod.
#[derive(Debug, Clone)]
pub struct DocToHubMsg(pub(crate) DocToHubMsgPayload);

#[derive(Debug, Clone)]
pub(crate) enum DocToHubMsgPayload {
    DocumentStatusChanged {
        new_status: DocumentStatus,
    },

    PeerStatesChanged {
        new_states: HashMap<ConnectionId, PeerDocState>,
    },
    /// Announce policy resolved to allow this connection to receive updates
    /// for this document, so hub retention should include a remote pin.
    RemotePinAcquired {
        connection_id: ConnectionId,
    },
    /// Connection is no longer receiving updates for this document, so the
    /// corresponding remote retention pin can be released.
    RemotePinReleased {
        connection_id: ConnectionId,
    },

    SendSyncMessage {
        connection_id: ConnectionId,
        document_id: DocumentId,
        message: SyncMessage,
    },

    Broadcast {
        connections: Vec<ConnectionId>,
        msg: Broadcast,
    },

    Terminated,
}

#[derive(Debug, Clone)]
pub enum Broadcast {
    New { msg: Vec<u8> },
    Gossip { msg: EphemeralMessage },
}
