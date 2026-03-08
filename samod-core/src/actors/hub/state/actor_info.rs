use std::collections::HashSet;

use crate::{
    ConnectionId, DocumentActorId, DocumentId, UnixTimestamp,
    actors::{document::DocumentStatus, local_repo::LocalRepoActorId},
};

#[derive(Debug, Clone)]
pub(crate) struct ActorInfo {
    pub(crate) actor_id: DocumentActorId,
    pub(crate) document_id: DocumentId,
    pub(crate) status: DocumentStatus,
    pub(crate) has_local_handles: bool,
    pub(crate) remote_pins: HashSet<ConnectionId>,
    pub(crate) local_handle_generation: u64,
    pub(crate) eviction_generation: u64,
    pub(crate) pending_eviction_deadline: Option<UnixTimestamp>,
    pub(crate) eviction_requested: bool,
}

impl ActorInfo {
    pub fn new_with_id(actor_id: DocumentActorId, document_id: DocumentId) -> Self {
        Self {
            actor_id,
            document_id,
            status: DocumentStatus::Spawned,
            has_local_handles: false,
            remote_pins: HashSet::new(),
            local_handle_generation: 0,
            eviction_generation: 0,
            pending_eviction_deadline: None,
            eviction_requested: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalRepoActorInfo {
    pub(crate) actor_id: LocalRepoActorId,
}
