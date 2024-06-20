use hex::{assets::Texture, component_manager::Component, nalgebra::Vector4};
use std::sync::Arc;

#[derive(Clone)]
pub struct Instance {
    pub texture: Arc<Texture>,
    pub color: Vector4<f32>,
    pub layer: i32,
    pub active: bool,
}

impl Instance {
    pub fn new(texture: Arc<Texture>, color: Vector4<f32>, layer: i32, active: bool) -> Self {
        Self {
            texture,
            color,
            layer,
            active,
        }
    }
}

impl Component for Instance {}
