use crate::actors::local_repo::LocalRepoActorId;

#[derive(Clone)]
pub struct SpawnArgs {
    pub(crate) actor_id: LocalRepoActorId,
}

impl std::fmt::Debug for SpawnArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnArgs")
            .field("actor_id", &self.actor_id)
            .finish()
    }
}

impl SpawnArgs {
    pub fn actor_id(&self) -> LocalRepoActorId {
        self.actor_id
    }
}
