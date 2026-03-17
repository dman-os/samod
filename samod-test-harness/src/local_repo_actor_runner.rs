use std::collections::{HashSet, VecDeque};

use samod_core::{
    actors::{
        HubToLocalRepoMsg, LocalRepoToHubMsg,
        local_repo::{
            LocalRepoActor, LocalRepoActorResult, SpawnArgs,
            io::{LocalRepoIoResult, LocalRepoIoTask},
        },
    },
    io::{IoResult, IoTask, IoTaskId},
};

use crate::Storage;

pub(crate) struct LocalRepoActorRunner {
    actor: LocalRepoActor,
    inbox: VecDeque<ActorEvent>,
    outbox: VecDeque<LocalRepoToHubMsg>,
    pending_storage_tasks: HashSet<IoTaskId>,
}

impl LocalRepoActorRunner {
    pub(crate) fn new(args: SpawnArgs) -> Self {
        let (actor, results) = LocalRepoActor::new(args);
        let mut runner = Self {
            actor,
            inbox: VecDeque::new(),
            outbox: VecDeque::new(),
            pending_storage_tasks: HashSet::new(),
        };
        runner.enqueue_events(results);
        runner
    }

    pub(crate) fn handle_events(&mut self, storage: &mut Storage) {
        self.handle_completed_storage(storage);
        while let Some(event) = self.inbox.pop_front() {
            if self.actor.is_stopped() {
                self.inbox.clear();
                return;
            }
            match event {
                ActorEvent::Message(msg) => {
                    let result = self.actor.handle_message(msg);
                    self.enqueue_events(result);
                }
                ActorEvent::Io(task) => {
                    match task.action {
                        LocalRepoIoTask::Storage(storage_task) => {
                            storage.handle_task(task.task_id, storage_task);
                            self.pending_storage_tasks.insert(task.task_id);
                        }
                    };
                }
            }
            self.handle_completed_storage(storage);
        }
        self.handle_completed_storage(storage);
    }

    fn handle_completed_storage(&mut self, storage: &mut Storage) {
        let mut completed = Vec::new();
        self.pending_storage_tasks.retain(|task_id| {
            let Some(result) = storage.check_pending_task(*task_id) else {
                return true;
            };
            completed.push((*task_id, result));
            false
        });

        for (task_id, result) in completed {
            let actor_result = self.actor.handle_io_complete(IoResult {
                task_id,
                payload: LocalRepoIoResult::Storage(result),
            });
            self.enqueue_events(actor_result);
        }
    }

    fn enqueue_events(&mut self, result: LocalRepoActorResult) {
        let LocalRepoActorResult {
            io_tasks,
            outgoing_messages,
            stopped: _,
        } = result;
        for task in io_tasks {
            self.inbox.push_back(ActorEvent::Io(task));
        }
        for msg in outgoing_messages {
            self.outbox.push_back(msg);
        }
    }

    pub(crate) fn deliver_message_to_inbox(&mut self, message: HubToLocalRepoMsg) {
        self.inbox.push_back(ActorEvent::Message(message));
    }

    pub(crate) fn take_outbox(&mut self) -> Vec<LocalRepoToHubMsg> {
        self.outbox.drain(..).collect()
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.actor.is_stopped()
    }
}

enum ActorEvent {
    Message(HubToLocalRepoMsg),
    Io(IoTask<LocalRepoIoTask>),
}
