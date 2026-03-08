use std::collections::HashMap;

use automerge::Automerge;

use crate::{
    DocumentActorId, DocumentId, StorageKey,
    actors::hub::{CommandId, CommandResult},
    io::{IoTaskId, StorageResult, StorageTask},
};

#[derive(Clone, Copy)]
enum PendingReadyKind {
    Create,
    Import,
}

struct PendingImportCheck {
    command_ids: Vec<CommandId>,
    content: Automerge,
    snapshot_task_id: IoTaskId,
    incremental_task_id: IoTaskId,
    snapshot_occupied: Option<bool>,
    incremental_occupied: Option<bool>,
}

pub(super) struct ImportReady {
    pub(super) document_id: DocumentId,
    pub(super) content: Automerge,
    pub(super) command_ids: Vec<CommandId>,
}

pub(super) enum ImportIoOutcome {
    Pending,
    Occupied {
        document_id: DocumentId,
        command_ids: Vec<CommandId>,
    },
    Ready(ImportReady),
}

pub(super) struct PendingCommands {
    pending_find_commands: HashMap<DocumentId, Vec<CommandId>>,
    pending_ready_commands: HashMap<DocumentActorId, Vec<(CommandId, PendingReadyKind)>>,
    pending_import_checks: HashMap<DocumentId, PendingImportCheck>,
    completed_commands: Vec<(CommandId, CommandResult)>,
}

impl PendingCommands {
    pub(super) fn new() -> Self {
        Self {
            pending_find_commands: HashMap::new(),
            pending_ready_commands: HashMap::new(),
            pending_import_checks: HashMap::new(),
            completed_commands: Vec::new(),
        }
    }

    pub(super) fn add_pending_find_command(
        &mut self,
        document_id: DocumentId,
        command_id: CommandId,
    ) {
        self.pending_find_commands
            .entry(document_id)
            .or_default()
            .push(command_id);
    }

    pub(super) fn add_pending_create_command(
        &mut self,
        actor_id: DocumentActorId,
        command_id: CommandId,
    ) {
        self.pending_ready_commands
            .entry(actor_id)
            .or_default()
            .push((command_id, PendingReadyKind::Create));
    }

    pub(super) fn add_pending_import_ready_commands(
        &mut self,
        actor_id: DocumentActorId,
        command_ids: Vec<CommandId>,
    ) {
        let pending = self.pending_ready_commands.entry(actor_id).or_default();
        pending.extend(
            command_ids
                .into_iter()
                .map(|command_id| (command_id, PendingReadyKind::Import)),
        );
    }

    pub(super) fn resolve_pending_ready(
        &mut self,
        actor_id: DocumentActorId,
        document_id: &DocumentId,
    ) {
        if let Some(command_ids) = self.pending_ready_commands.remove(&actor_id) {
            for (command_id, kind) in command_ids {
                let result = match kind {
                    PendingReadyKind::Create => CommandResult::CreateDocument {
                        actor_id,
                        document_id: document_id.clone(),
                    },
                    PendingReadyKind::Import => CommandResult::ImportDocument {
                        actor_id,
                        document_id: document_id.clone(),
                    },
                };
                self.completed_commands.push((command_id, result));
            }
        }
    }

    pub(super) fn resolve_pending_find(
        &mut self,
        document_id: &DocumentId,
        actor_id: DocumentActorId,
        found: bool,
    ) {
        if let Some(command_ids) = self.pending_find_commands.remove(document_id) {
            for command_id in command_ids {
                self.completed_commands
                    .push((command_id, CommandResult::FindDocument { actor_id, found }));
            }
        }
    }

    pub(super) fn has_pending_create(&self, doc_actor_id: DocumentActorId) -> bool {
        self.pending_ready_commands.contains_key(&doc_actor_id)
    }

    pub(super) fn start_pending_import_check(
        &mut self,
        document_id: DocumentId,
        command_id: CommandId,
        content: Automerge,
        snapshot_task_id: IoTaskId,
        incremental_task_id: IoTaskId,
    ) {
        self.pending_import_checks.insert(
            document_id,
            PendingImportCheck {
                command_ids: vec![command_id],
                content,
                snapshot_task_id,
                incremental_task_id,
                snapshot_occupied: None,
                incremental_occupied: None,
            },
        );
    }

    pub(super) fn add_pending_import_command(
        &mut self,
        document_id: &DocumentId,
        command_id: CommandId,
    ) -> bool {
        if let Some(check) = self.pending_import_checks.get_mut(document_id) {
            check.command_ids.push(command_id);
            true
        } else {
            false
        }
    }

    pub(super) fn handle_import_storage_result(
        &mut self,
        task_id: IoTaskId,
        result: StorageResult,
    ) -> Option<ImportIoOutcome> {
        let (document_id, occupied) =
            self.pending_import_checks
                .iter_mut()
                .find_map(|(document_id, check)| {
                    let StorageResult::LoadRange { ref values } = result else {
                        return None;
                    };

                    if check.snapshot_task_id == task_id {
                        check.snapshot_occupied = Some(!values.is_empty());
                        Some((document_id.clone(), !values.is_empty()))
                    } else if check.incremental_task_id == task_id {
                        check.incremental_occupied = Some(!values.is_empty());
                        Some((document_id.clone(), !values.is_empty()))
                    } else {
                        None
                    }
                })?;

        let check = self
            .pending_import_checks
            .get(&document_id)
            .expect("pending import check must exist");

        let (Some(snapshot_occupied), Some(incremental_occupied)) =
            (check.snapshot_occupied, check.incremental_occupied)
        else {
            return Some(ImportIoOutcome::Pending);
        };

        let check = self
            .pending_import_checks
            .remove(&document_id)
            .expect("pending import check must exist");

        if snapshot_occupied || incremental_occupied || occupied {
            Some(ImportIoOutcome::Occupied {
                document_id,
                command_ids: check.command_ids,
            })
        } else {
            Some(ImportIoOutcome::Ready(ImportReady {
                document_id,
                content: check.content,
                command_ids: check.command_ids,
            }))
        }
    }

    pub(super) fn complete_import_already_exists(
        &mut self,
        command_ids: Vec<CommandId>,
        document_id: DocumentId,
    ) {
        for command_id in command_ids {
            self.completed_commands.push((
                command_id,
                CommandResult::ImportDocumentAlreadyExists {
                    document_id: document_id.clone(),
                },
            ));
        }
    }

    pub(super) fn complete_single_import_already_exists(
        &mut self,
        command_id: CommandId,
        document_id: DocumentId,
    ) {
        self.completed_commands.push((
            command_id,
            CommandResult::ImportDocumentAlreadyExists { document_id },
        ));
    }

    pub(super) fn pop_completed_commands(&mut self) -> Vec<(CommandId, CommandResult)> {
        std::mem::take(&mut self.completed_commands)
    }
}

pub(super) fn import_storage_tasks(document_id: &DocumentId) -> [StorageTask; 2] {
    [
        StorageTask::LoadRange {
            prefix: StorageKey::snapshot_prefix(document_id),
        },
        StorageTask::LoadRange {
            prefix: StorageKey::incremental_prefix(document_id),
        },
    ]
}
