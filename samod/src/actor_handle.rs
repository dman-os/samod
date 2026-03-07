use std::sync::{Arc, Mutex, Weak};

use crate::{DocActorInner, doc_lease::DocLease};

pub(crate) struct ActorHandle {
    pub(crate) inner: Arc<Mutex<DocActorInner>>,
    pub(crate) lease: Weak<DocLease>,
    pub(crate) lease_generation: u64,
}
