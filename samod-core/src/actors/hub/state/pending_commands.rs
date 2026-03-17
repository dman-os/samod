use std::collections::HashMap;

use crate::{
    DocumentActorId, DocumentId,
    actors::hub::{CommandId, CommandResult},
};

#[derive(Clone, Copy)]
enum PendingReadyKind {
    Create,
    Import,
}

pub(super) struct PendingCommands {
    pending_find_commands: HashMap<DocumentId, Vec<CommandId>>,
    pending_ready_commands: HashMap<DocumentActorId, Vec<(CommandId, PendingReadyKind)>>,
    completed_commands: Vec<(CommandId, CommandResult)>,
}

impl PendingCommands {
    pub(super) fn new() -> Self {
        Self {
            pending_find_commands: HashMap::new(),
            pending_ready_commands: HashMap::new(),
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
