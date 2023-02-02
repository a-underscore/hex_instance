use hex::{cid, ecs::component_manager::Component, id::id};
use std::{rc::Rc, sync::atomic::AtomicUsize};

pub fn iid() -> Rc<usize> {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    Rc::new(id(&COUNT))
}

#[derive(Clone)]
pub struct InstanceId {
    id: Rc<usize>,
}

impl InstanceId {
    pub fn get(&self) -> usize {
        *self.id
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self { id: iid() }
    }
}

impl Component for InstanceId {
    fn id() -> usize {
        cid!()
    }
}
