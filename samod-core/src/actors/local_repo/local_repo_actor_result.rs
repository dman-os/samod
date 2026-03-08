use crate::{
    actors::{
        LocalRepoToHubMsg, local_repo::io::LocalRepoIoTask, messages::LocalRepoToHubMsgPayload,
    },
    io::{IoTask, IoTaskId, StorageTask},
};

#[derive(Debug, Default)]
pub struct LocalRepoActorResult {
    pub io_tasks: Vec<IoTask<LocalRepoIoTask>>,
    pub outgoing_messages: Vec<LocalRepoToHubMsg>,
    pub stopped: bool,
}

impl LocalRepoActorResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn send_message(&mut self, message: LocalRepoToHubMsgPayload) {
        self.outgoing_messages.push(LocalRepoToHubMsg(message));
    }

    pub(crate) fn enqueue_storage_task(&mut self, task: StorageTask) -> IoTaskId {
        let io_task = IoTask::new(LocalRepoIoTask::Storage(task));
        let task_id = io_task.task_id;
        self.io_tasks.push(io_task);
        task_id
    }
}
