use hex::{cid, ecs::component_manager::Component, id::id};
use std::sync::atomic::AtomicUsize;

pub fn iid() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    id(&COUNT)
}

#[derive(Clone)]
pub struct Instance {
    id: usize,
}

impl Instance {
    pub fn get(&self) -> usize {
        self.id
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self { id: iid() }
    }
}

impl Component for Instance {
    fn id() -> usize {
        cid!()
    }
}
