use hex::{
    assets::{Shape, Texture},
    cid,
    ecs::component_manager::Component,
    glium::{
        draw_parameters::{Blend, DepthTest},
        Depth, DrawParameters,
    },
    id,
};
use std::sync::atomic::AtomicUsize;

pub fn iid() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    id(&COUNT)
}

#[derive(Clone)]
pub struct Instance<'a> {
    pub draw_parameters: DrawParameters<'a>,
    pub shape: Shape,
    pub texture: Texture,
    pub color: [f32; 4],
    pub z: f32,
    pub active: bool,
    id: usize,
}

impl<'a> Instance<'a> {
    pub fn new(shape: Shape, texture: Texture, color: [f32; 4], z: f32, active: bool) -> Self {
        Self {
            draw_parameters: DrawParameters {
                depth: Depth {
                    test: DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: Blend::alpha_blending(),
                ..Default::default()
            },
            shape,
            texture,
            color,
            z,
            active,
            id: iid(),
        }
    }

    pub fn get(&self) -> usize {
        self.id
    }
}

impl<'a> Component for Instance<'a> {
    fn id() -> usize {
        cid!()
    }
}
