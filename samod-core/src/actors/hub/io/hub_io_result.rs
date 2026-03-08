use crate::io::StorageResult;

#[derive(Debug, Clone)]
pub enum HubIoResult {
    Send,
    Disconnect,
    Storage(StorageResult),
}
