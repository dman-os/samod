use crate::{DocumentId, actors::local_repo::LocalRepoRequestId};

#[derive(Debug, Clone)]
pub struct LocalRepoToHubMsg(pub(crate) LocalRepoToHubMsgPayload);

#[derive(Debug, Clone)]
pub(crate) enum LocalRepoToHubMsgPayload {
    ImportPreflightComplete {
        request_id: LocalRepoRequestId,
        document_id: DocumentId,
        occupied: bool,
    },
    Terminated,
}
