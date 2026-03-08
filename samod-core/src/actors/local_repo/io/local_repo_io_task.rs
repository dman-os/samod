use crate::io::StorageTask;

#[derive(Debug)]
pub enum LocalRepoIoTask {
    Storage(StorageTask),
}
