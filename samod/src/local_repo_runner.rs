use std::sync::{Arc, Mutex};

use futures::{FutureExt, StreamExt, stream::FuturesUnordered};
use samod_core::actors::local_repo::{LocalRepoActorId, LocalRepoActorResult};

use crate::{
    local_repo_actor_inner::LocalRepoActorInner, local_repo_actor_task::LocalRepoActorTask,
    unbounded::UnboundedReceiver,
};

pub(crate) struct SpawnedLocalRepoActor {
    pub(crate) actor_id: LocalRepoActorId,
    pub(crate) inner: Arc<Mutex<LocalRepoActorInner>>,
    pub(crate) rx_tasks: UnboundedReceiver<LocalRepoActorTask>,
    pub(crate) init_results: LocalRepoActorResult,
}

pub(crate) async fn async_local_repo_actor_runner(rx: UnboundedReceiver<SpawnedLocalRepoActor>) {
    let mut running_actors = FuturesUnordered::new();

    loop {
        futures::select! {
            spawn_actor = rx.recv().fuse() => {
                match spawn_actor {
                    Err(_) => break,
                    Ok(SpawnedLocalRepoActor { inner, rx_tasks, init_results, actor_id }) => {
                        running_actors.push(async move {
                            inner.lock().unwrap().handle_results(init_results);
                            while let Ok(actor_task) = rx_tasks.recv().await {
                                let mut inner = inner.lock().unwrap();
                                inner.handle_task(actor_task);
                                if inner.is_stopped() {
                                    tracing::debug!(?actor_id, "local repo actor stopped");
                                    break;
                                }
                            }
                        });
                    }
                }
            },
            _ = running_actors.next() => {}
        }
    }

    while running_actors.next().await.is_some() {}
}
