use samod_core::{
    actors::{HubToLocalRepoMsg, local_repo::io::LocalRepoIoResult},
    io::IoResult,
};

#[derive(Debug)]
pub(crate) enum LocalRepoActorTask {
    HandleMessage(HubToLocalRepoMsg),
    IoComplete(IoResult<LocalRepoIoResult>),
}
