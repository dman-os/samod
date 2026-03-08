use std::collections::HashMap;

use crate::{
    DocumentId,
    actors::{
        HubToLocalRepoMsg, RunState,
        local_repo::{
            LocalRepoActorId, LocalRepoActorResult, LocalRepoRequestId, SpawnArgs,
            io::LocalRepoIoResult,
        },
        messages::{HubToLocalRepoMsgPayload, LocalRepoToHubMsgPayload},
    },
    io::{IoResult, IoTaskId, StorageResult, StorageTask},
};

struct InflightImportCheck {
    request_ids: Vec<LocalRepoRequestId>,
    snapshot_task_id: IoTaskId,
    incremental_task_id: IoTaskId,
    snapshot_occupied: Option<bool>,
    incremental_occupied: Option<bool>,
}

pub struct LocalRepoActor {
    id: LocalRepoActorId,
    inflight_import_checks: HashMap<DocumentId, InflightImportCheck>,
    run_state: RunState,
}

impl LocalRepoActor {
    pub fn new(SpawnArgs { actor_id }: SpawnArgs) -> (Self, LocalRepoActorResult) {
        (
            Self {
                id: actor_id,
                inflight_import_checks: HashMap::new(),
                run_state: RunState::Running,
            },
            LocalRepoActorResult::default(),
        )
    }

    pub fn handle_message(&mut self, message: HubToLocalRepoMsg) -> LocalRepoActorResult {
        let mut out = LocalRepoActorResult::default();
        match message.0 {
            HubToLocalRepoMsgPayload::Terminate => {
                self.run_state = RunState::Stopped;
                out.stopped = true;
                out.send_message(LocalRepoToHubMsgPayload::Terminated);
            }
            HubToLocalRepoMsgPayload::ImportPreflight {
                document_id,
                request_id,
            } => {
                if let Some(inflight) = self.inflight_import_checks.get_mut(&document_id) {
                    inflight.request_ids.push(request_id);
                    return out;
                }

                let snapshot_task_id = out.enqueue_storage_task(StorageTask::LoadRange {
                    prefix: crate::StorageKey::snapshot_prefix(&document_id),
                });
                let incremental_task_id = out.enqueue_storage_task(StorageTask::LoadRange {
                    prefix: crate::StorageKey::incremental_prefix(&document_id),
                });
                self.inflight_import_checks.insert(
                    document_id,
                    InflightImportCheck {
                        request_ids: vec![request_id],
                        snapshot_task_id,
                        incremental_task_id,
                        snapshot_occupied: None,
                        incremental_occupied: None,
                    },
                );
            }
        }
        out
    }

    pub fn handle_io_complete(
        &mut self,
        io_result: IoResult<LocalRepoIoResult>,
    ) -> LocalRepoActorResult {
        let mut out = LocalRepoActorResult::default();
        let IoResult { task_id, payload } = io_result;
        let LocalRepoIoResult::Storage(StorageResult::LoadRange { values }) = payload else {
            panic!("unexpected local repo IO result");
        };

        let Some(document_id) =
            self.inflight_import_checks
                .iter_mut()
                .find_map(|(document_id, inflight)| {
                    if inflight.snapshot_task_id == task_id {
                        inflight.snapshot_occupied = Some(!values.is_empty());
                        Some(document_id.clone())
                    } else if inflight.incremental_task_id == task_id {
                        inflight.incremental_occupied = Some(!values.is_empty());
                        Some(document_id.clone())
                    } else {
                        None
                    }
                })
        else {
            panic!("unknown task ID in local repo actor {}", self.id);
        };

        let ready = {
            let inflight = self
                .inflight_import_checks
                .get(&document_id)
                .expect("missing inflight import check");
            match (inflight.snapshot_occupied, inflight.incremental_occupied) {
                (Some(snapshot), Some(incremental)) => Some(snapshot || incremental),
                _ => None,
            }
        };

        if let Some(occupied) = ready {
            let inflight = self
                .inflight_import_checks
                .remove(&document_id)
                .expect("missing inflight import check");
            for request_id in inflight.request_ids {
                out.send_message(LocalRepoToHubMsgPayload::ImportPreflightComplete {
                    request_id,
                    document_id: document_id.clone(),
                    occupied,
                });
            }
        }

        out
    }

    pub fn is_stopped(&self) -> bool {
        self.run_state == RunState::Stopped
    }
}
