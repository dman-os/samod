use samod_core::DocumentId;

use crate::unbounded::UnboundedSender;

#[derive(Debug, Clone)]
pub(crate) enum LeaseNotification {
    Dropped {
        document_id: DocumentId,
        generation: u64,
    },
}

pub(crate) struct DocLease {
    document_id: DocumentId,
    generation: u64,
    tx_repo_events: UnboundedSender<LeaseNotification>,
}

impl DocLease {
    pub(crate) fn new(
        document_id: DocumentId,
        generation: u64,
        tx_repo_events: UnboundedSender<LeaseNotification>,
    ) -> Self {
        Self {
            document_id,
            generation,
            tx_repo_events,
        }
    }
}

impl Drop for DocLease {
    fn drop(&mut self) {
        let _ = self
            .tx_repo_events
            .unbounded_send(LeaseNotification::Dropped {
                document_id: self.document_id.clone(),
                generation: self.generation,
            });
    }
}
