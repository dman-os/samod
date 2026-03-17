use std::sync::Arc;

use samod_core::{
    actors::{
        LocalRepoToHubMsg,
        local_repo::{LocalRepoActor, LocalRepoActorId, LocalRepoActorResult, io::LocalRepoIoTask},
    },
    io::IoTask,
};

use crate::{
    io_loop::IoLoopTask, local_repo_actor_task::LocalRepoActorTask, observer::RepoObserver,
    unbounded::UnboundedSender,
};

pub(crate) struct LocalRepoActorInner {
    actor_id: LocalRepoActorId,
    tx_to_core: UnboundedSender<(LocalRepoActorId, LocalRepoToHubMsg)>,
    tx_io: UnboundedSender<IoLoopTask>,
    actor: LocalRepoActor,
    observer: Option<Arc<dyn RepoObserver>>,
}

impl LocalRepoActorInner {
    pub(crate) fn new(
        actor_id: LocalRepoActorId,
        actor: LocalRepoActor,
        tx_to_core: UnboundedSender<(LocalRepoActorId, LocalRepoToHubMsg)>,
        tx_io: UnboundedSender<IoLoopTask>,
        observer: Option<Arc<dyn RepoObserver>>,
    ) -> Self {
        Self {
            actor_id,
            tx_to_core,
            tx_io,
            actor,
            observer,
        }
    }

    pub(crate) fn handle_results(&mut self, results: LocalRepoActorResult) {
        let LocalRepoActorResult {
            io_tasks,
            outgoing_messages,
            stopped,
        } = results;

        let _ = &self.observer;
        let _ = stopped;

        for task in io_tasks {
            let mapped = IoTask {
                task_id: task.task_id,
                action: match task.action {
                    LocalRepoIoTask::Storage(storage_task) => storage_task,
                },
            };
            if self
                .tx_io
                .unbounded_send(IoLoopTask::LocalRepoStorage {
                    task: mapped,
                    actor_id: self.actor_id,
                })
                .is_err()
            {
                tracing::error!(?self.actor_id, "io receiver dropped whilst local repo actor is running");
                return;
            }
        }

        for msg in outgoing_messages {
            if self
                .tx_to_core
                .unbounded_send((self.actor_id, msg))
                .is_err()
            {
                tracing::error!(?self.actor_id, "core receiver dropped whilst local repo actor is running");
                return;
            }
        }
    }

    pub(crate) fn handle_task(&mut self, task: LocalRepoActorTask) {
        let result = match task {
            LocalRepoActorTask::HandleMessage(message) => self.actor.handle_message(message),
            LocalRepoActorTask::IoComplete(io_result) => self.actor.handle_io_complete(io_result),
        };
        self.handle_results(result);
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.actor.is_stopped()
    }
}
