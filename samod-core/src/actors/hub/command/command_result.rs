use crate::{ConnectionId, DialerId, DocumentActorId, DocumentId, ListenerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResult {
    /// Connection created, handshake initiated if outgoing.
    CreateConnection {
        connection_id: ConnectionId,
    },
    DisconnectConnection,
    /// Message received and processed.
    Receive {
        connection_id: ConnectionId,
        /// Any protocol errors that occurred
        error: Option<String>,
    },
    /// Result of ActorReady command.
    ActorReady,
    /// Result of CreateDocument command.
    CreateDocument {
        actor_id: DocumentActorId,
        document_id: DocumentId,
    },
    /// Result of ImportDocument command.
    ImportDocument {
        actor_id: DocumentActorId,
        document_id: DocumentId,
    },
    ImportDocumentAlreadyExists {
        document_id: DocumentId,
    },
    ExportDocumentLocal {
        document_id: DocumentId,
        bytes: Vec<u8>,
    },
    ExportDocumentLocalNotFound {
        document_id: DocumentId,
    },
    ContainsDocumentLocal {
        document_id: DocumentId,
        contains: bool,
    },
    /// Result of FindDocument command.
    FindDocument {
        actor_id: DocumentActorId,
        found: bool,
    },
    /// Result of AddDialer command.
    AddDialer {
        dialer_id: DialerId,
    },
    /// Result of AddListener command.
    AddListener {
        listener_id: ListenerId,
    },
}
