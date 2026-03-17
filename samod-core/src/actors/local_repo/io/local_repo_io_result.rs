use crate::io::StorageResult;

#[derive(Debug, Clone)]
pub enum LocalRepoIoResult {
    Storage(StorageResult),
}
