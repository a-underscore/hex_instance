use hex::{assets::Texture, cid, ecs::component_manager::Component};
use std::rc::Rc;

#[derive(Clone)]
pub struct Instance {
    pub texture: Rc<Texture>,
    pub color: [f32; 4],
    pub z: f32,
    pub active: bool,
}

impl Instance {
    pub fn new(texture: Rc<Texture>, color: [f32; 4], z: f32, active: bool) -> Self {
        Self {
            texture,
            color,
            z,
            active,
        }
    }
}

impl Component for Instance {
    fn id() -> usize {
        cid!()
    }
}
