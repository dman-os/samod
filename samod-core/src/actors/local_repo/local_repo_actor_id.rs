use std::sync::atomic::{AtomicU32, Ordering};

static LAST_LOCAL_REPO_ACTOR_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalRepoActorId(u32);

impl Default for LocalRepoActorId {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalRepoActorId {
    pub fn new() -> Self {
        let id = LAST_LOCAL_REPO_ACTOR_ID.fetch_add(1, Ordering::SeqCst);
        LocalRepoActorId(id)
    }
}

impl std::fmt::Display for LocalRepoActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "local-repo-actor:{}", self.0)
    }
}
