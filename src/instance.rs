use hex::{
    assets::Texture,
    cid,
    ecs::component_manager::Component,
    glium::{
        draw_parameters::{Blend, DepthTest},
        Depth, DrawParameters,
    },
};

#[derive(Clone)]
pub struct Instance<'a> {
    pub draw_parameters: DrawParameters<'a>,
    pub texture: Texture,
    pub color: [f32; 4],
    pub z: f32,
    pub id: usize,
    pub active: bool,
}

impl<'a> Instance<'a> {
    pub fn new(texture: Texture, color: [f32; 4], z: f32, id: usize, active: bool) -> Self {
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
            texture,
            color,
            z,
            id,
            active,
        }
    }
}

impl<'a> Component for Instance<'a> {
    fn id() -> usize {
        cid!()
    }
}
