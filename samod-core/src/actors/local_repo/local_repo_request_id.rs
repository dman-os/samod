use std::sync::atomic::{AtomicU32, Ordering};

static LAST_LOCAL_REPO_REQUEST_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalRepoRequestId(u32);

impl LocalRepoRequestId {
    pub fn new() -> Self {
        let id = LAST_LOCAL_REPO_REQUEST_ID.fetch_add(1, Ordering::SeqCst);
        Self(id)
    }
}
