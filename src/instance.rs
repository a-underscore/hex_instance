use hex::{cid, ecs::component_manager::Component, glium::vertex::PerInstance};
use std::rc::Rc;

#[derive(Clone)]
pub struct Instance<'a>(pub Rc<PerInstance<'a>>);

impl Component for Instance<'_> {
    fn id() -> usize {
        cid!()
    }
}
