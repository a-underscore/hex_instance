use hex::{
    assets::Shape,
    cgmath::Vector4,
    cid,
    ecs::{component_manager::Component, id::id},
    glium::{
        draw_parameters::{Blend, DepthTest},
        texture::RawImage2d,
        uniforms::SamplerBehavior,
        Depth, DrawParameters,
    },
};
use std::{rc::Rc, sync::atomic::AtomicUsize};

pub fn iid() -> usize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    id(&COUNT)
}

#[derive(Clone)]
pub struct Instance<'a, 'b> {
    pub draw_parameters: DrawParameters<'a>,
    pub shape: Shape,
    pub texture: Rc<RawImage2d<'b, u8>>,
    pub sampler_behaviour: SamplerBehavior,
    pub color: Vector4<f32>,
    pub z: f32,
    pub active: bool,
    id: usize,
}

impl<'a, 'b> Instance<'a, 'b> {
    pub fn new(
        shape: Shape,
        texture: Rc<RawImage2d<'b, u8>>,
        sampler_behaviour: SamplerBehavior,
        color: Vector4<f32>,
        z: f32,
        active: bool,
    ) -> Self {
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
            sampler_behaviour,
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

impl Component for Instance<'_, '_> {
    fn id() -> usize {
        cid!()
    }
}
