use hex::{assets::Texture2d, ecs::component_manager::Component};
use std::sync::Arc;

#[derive(Clone)]
pub struct Instance {
    pub texture: Arc<Texture2d>,
    pub color: [f32; 4],
    pub z: f32,
    pub active: bool,
}

impl Instance {
    pub fn new(texture: Texture2d, color: [f32; 4], z: f32, active: bool) -> Self {
        Self {
            texture: Arc::new(texture),
            color,
            z,
            active,
        }
    }
}

impl Component for Instance {}
