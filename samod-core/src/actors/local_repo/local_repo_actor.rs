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

struct InflightContainsCheck {
    request_ids: Vec<LocalRepoRequestId>,
    snapshot_task_id: IoTaskId,
    incremental_task_id: IoTaskId,
    snapshot_occupied: Option<bool>,
    incremental_occupied: Option<bool>,
}

struct InflightExportLoad {
    request_ids: Vec<LocalRepoRequestId>,
    snapshot_task_id: IoTaskId,
    incremental_task_id: IoTaskId,
    snapshot_values: Option<HashMap<crate::StorageKey, Vec<u8>>>,
    incremental_values: Option<HashMap<crate::StorageKey, Vec<u8>>>,
}

pub struct LocalRepoActor {
    id: LocalRepoActorId,
    inflight_import_checks: HashMap<DocumentId, InflightImportCheck>,
    inflight_contains_checks: HashMap<DocumentId, InflightContainsCheck>,
    inflight_export_loads: HashMap<DocumentId, InflightExportLoad>,
    run_state: RunState,
}

impl LocalRepoActor {
    pub fn new(SpawnArgs { actor_id }: SpawnArgs) -> (Self, LocalRepoActorResult) {
        (
            Self {
                id: actor_id,
                inflight_import_checks: HashMap::new(),
                inflight_contains_checks: HashMap::new(),
                inflight_export_loads: HashMap::new(),
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
            HubToLocalRepoMsgPayload::ContainsDocument {
                document_id,
                request_id,
            } => {
                if let Some(inflight) = self.inflight_contains_checks.get_mut(&document_id) {
                    inflight.request_ids.push(request_id);
                    return out;
                }

                let snapshot_task_id = out.enqueue_storage_task(StorageTask::LoadRange {
                    prefix: crate::StorageKey::snapshot_prefix(&document_id),
                });
                let incremental_task_id = out.enqueue_storage_task(StorageTask::LoadRange {
                    prefix: crate::StorageKey::incremental_prefix(&document_id),
                });
                self.inflight_contains_checks.insert(
                    document_id,
                    InflightContainsCheck {
                        request_ids: vec![request_id],
                        snapshot_task_id,
                        incremental_task_id,
                        snapshot_occupied: None,
                        incremental_occupied: None,
                    },
                );
            }
            HubToLocalRepoMsgPayload::ExportDocument {
                document_id,
                request_id,
            } => {
                if let Some(inflight) = self.inflight_export_loads.get_mut(&document_id) {
                    inflight.request_ids.push(request_id);
                    return out;
                }

                let snapshot_task_id = out.enqueue_storage_task(StorageTask::LoadRange {
                    prefix: crate::StorageKey::snapshot_prefix(&document_id),
                });
                let incremental_task_id = out.enqueue_storage_task(StorageTask::LoadRange {
                    prefix: crate::StorageKey::incremental_prefix(&document_id),
                });
                self.inflight_export_loads.insert(
                    document_id,
                    InflightExportLoad {
                        request_ids: vec![request_id],
                        snapshot_task_id,
                        incremental_task_id,
                        snapshot_values: None,
                        incremental_values: None,
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

        if self.complete_import_preflight(task_id, &values, &mut out) {
            return out;
        }
        if self.complete_contains(task_id, &values, &mut out) {
            return out;
        }
        if self.complete_export(task_id, Some(values), &mut out) {
            return out;
        }
        panic!("unknown task ID in local repo actor {}", self.id);
    }

    pub fn is_stopped(&self) -> bool {
        self.run_state == RunState::Stopped
    }

    fn complete_import_preflight(
        &mut self,
        task_id: IoTaskId,
        values: &HashMap<crate::StorageKey, Vec<u8>>,
        out: &mut LocalRepoActorResult,
    ) -> bool {
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
            return false;
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
        true
    }

    fn complete_contains(
        &mut self,
        task_id: IoTaskId,
        values: &HashMap<crate::StorageKey, Vec<u8>>,
        out: &mut LocalRepoActorResult,
    ) -> bool {
        let Some(document_id) =
            self.inflight_contains_checks
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
            return false;
        };

        let ready = {
            let inflight = self
                .inflight_contains_checks
                .get(&document_id)
                .expect("missing inflight contains check");
            match (inflight.snapshot_occupied, inflight.incremental_occupied) {
                (Some(snapshot), Some(incremental)) => Some(snapshot || incremental),
                _ => None,
            }
        };

        if let Some(contains) = ready {
            let inflight = self
                .inflight_contains_checks
                .remove(&document_id)
                .expect("missing inflight contains check");
            for request_id in inflight.request_ids {
                out.send_message(LocalRepoToHubMsgPayload::ContainsDocumentComplete {
                    request_id,
                    document_id: document_id.clone(),
                    contains,
                });
            }
        }
        true
    }

    fn complete_export(
        &mut self,
        task_id: IoTaskId,
        mut values: Option<HashMap<crate::StorageKey, Vec<u8>>>,
        out: &mut LocalRepoActorResult,
    ) -> bool {
        let Some(document_id) =
            self.inflight_export_loads
                .iter_mut()
                .find_map(|(document_id, inflight)| {
                    if inflight.snapshot_task_id == task_id {
                        inflight.snapshot_values = values.take();
                        Some(document_id.clone())
                    } else if inflight.incremental_task_id == task_id {
                        inflight.incremental_values = values.take();
                        Some(document_id.clone())
                    } else {
                        None
                    }
                })
        else {
            return false;
        };

        let ready = self
            .inflight_export_loads
            .get(&document_id)
            .is_some_and(|inflight| {
                inflight.snapshot_values.is_some() && inflight.incremental_values.is_some()
            });
        if !ready {
            return true;
        }

        let inflight = self
            .inflight_export_loads
            .remove(&document_id)
            .expect("missing inflight export load");
        let snapshot_values = inflight.snapshot_values.unwrap_or_default();
        let incremental_values = inflight.incremental_values.unwrap_or_default();
        let bytes = if snapshot_values.is_empty() && incremental_values.is_empty() {
            None
        } else {
            let mut doc = automerge::Automerge::new();
            for (key, value) in &snapshot_values {
                if let Err(err) = doc.load_incremental(value) {
                    tracing::warn!(%key, ?err, "error loading snapshot chunk for local export");
                }
            }
            for (key, value) in &incremental_values {
                if let Err(err) = doc.load_incremental(value) {
                    tracing::warn!(%key, ?err, "error loading incremental chunk for local export");
                }
            }
            Some(doc.save())
        };
        for request_id in inflight.request_ids {
            out.send_message(LocalRepoToHubMsgPayload::ExportDocumentComplete {
                request_id,
                document_id: document_id.clone(),
                bytes: bytes.clone(),
            });
        }
        true
    }
}
