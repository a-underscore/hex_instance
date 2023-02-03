use hex::{cid, components::Sprite, ecs::component_manager::Component, id::id};
use std::sync::atomic::AtomicUsize;

pub fn iid() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    id(&COUNT)
}

#[derive(Clone)]
pub struct Instance<'a> {
    pub sprite: Sprite<'a>,
    id: usize,
}

impl<'a> Instance<'a> {
    pub fn new(sprite: Sprite<'a>) -> Self {
        Self { sprite, id: iid() }
    }

    pub fn get(&self) -> usize {
        self.id
    }
}

impl Component for Instance<'_> {
    fn id() -> usize {
        cid!()
    }
}
