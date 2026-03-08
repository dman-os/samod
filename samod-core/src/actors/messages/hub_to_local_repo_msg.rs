use crate::{DocumentId, actors::local_repo::LocalRepoRequestId};

#[derive(Debug, Clone)]
pub struct HubToLocalRepoMsg(pub(crate) HubToLocalRepoMsgPayload);

#[derive(Debug, Clone)]
pub(crate) enum HubToLocalRepoMsgPayload {
    Terminate,
    ImportPreflight {
        document_id: DocumentId,
        request_id: LocalRepoRequestId,
    },
}
