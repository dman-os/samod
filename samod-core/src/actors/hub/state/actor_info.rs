use crate::UnixTimestamp;
use crate::{DocumentActorId, DocumentId, actors::document::DocumentStatus};

#[derive(Debug, Clone)]
pub(crate) struct ActorInfo {
    pub(crate) actor_id: DocumentActorId,
    pub(crate) document_id: DocumentId,
    pub(crate) status: DocumentStatus,
    pub(crate) has_local_handles: bool,
    pub(crate) local_handle_generation: u64,
    pub(crate) last_local_handle_drop_at: Option<UnixTimestamp>,
    pub(crate) eviction_requested: bool,
}

impl ActorInfo {
    pub fn new_with_id(actor_id: DocumentActorId, document_id: DocumentId) -> Self {
        Self {
            actor_id,
            document_id,
            status: DocumentStatus::Spawned,
            has_local_handles: false,
            local_handle_generation: 0,
            last_local_handle_drop_at: None,
            eviction_requested: false,
        }
    }
}
