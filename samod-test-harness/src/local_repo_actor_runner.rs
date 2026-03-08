use std::collections::VecDeque;

use samod_core::{
    actors::{
        HubToLocalRepoMsg, LocalRepoToHubMsg,
        local_repo::{
            LocalRepoActor, LocalRepoActorResult, SpawnArgs,
            io::{LocalRepoIoResult, LocalRepoIoTask},
        },
    },
    io::{IoResult, IoTask},
};

use crate::Storage;

pub(crate) struct LocalRepoActorRunner {
    actor: LocalRepoActor,
    inbox: VecDeque<ActorEvent>,
    outbox: VecDeque<LocalRepoToHubMsg>,
}

impl LocalRepoActorRunner {
    pub(crate) fn new(args: SpawnArgs) -> Self {
        let (actor, results) = LocalRepoActor::new(args);
        let mut runner = Self {
            actor,
            inbox: VecDeque::new(),
            outbox: VecDeque::new(),
        };
        runner.enqueue_events(results);
        runner
    }

    pub(crate) fn handle_events(&mut self, storage: &mut Storage) {
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
                    let io_result = match task.action {
                        LocalRepoIoTask::Storage(storage_task) => IoResult {
                            task_id: task.task_id,
                            payload: LocalRepoIoResult::Storage(storage.handle_task(storage_task)),
                        },
                    };
                    let actor_result = self.actor.handle_io_complete(io_result);
                    self.enqueue_events(actor_result);
                }
            }
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
