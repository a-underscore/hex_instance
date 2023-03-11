use crate::{Instance, InstanceData};
use hex::{
    anyhow,
    assets::Shader,
    components::{Camera, Transform},
    glium::{
        glutin::event::Event, index::NoIndices, uniform, uniforms::Sampler, Display, Surface,
        VertexBuffer,
    },
    hecs::{ev::Control, system_manager::System, Ev, World},
};
use std::collections::BTreeMap;

pub const INSTANCE_VERTEX_SRC: &str = include_str!("instance_vertex.glsl");
pub const INSTANCE_FRAGMENT_SRC: &str = include_str!("instance_fragment.glsl");

pub struct InstanceRenderer {
    pub shader: Shader,
}

impl InstanceRenderer {
    pub fn new(display: &Display) -> anyhow::Result<Self> {
        Ok(Self {
            shader: Shader::new(display, INSTANCE_VERTEX_SRC, INSTANCE_FRAGMENT_SRC, None)?,
        })
    }
}

impl<'a> System<'a> for InstanceRenderer {
    fn update(&mut self, event: &mut Ev, world: &mut World<'a>) -> anyhow::Result<()> {
        if let Ev::Draw((
            Control {
                event: Event::MainEventsCleared,
                flow: _,
            },
            target,
        )) = event
        {
            if let Some((c, ct)) = world.em.entities.keys().cloned().find_map(|e| {
                Some((
                    world
                        .cm
                        .get::<Camera>(e, &world.em)
                        .and_then(|c| c.active.then_some(c))?,
                    world
                        .cm
                        .get::<Transform>(e, &world.em)
                        .and_then(|t| t.active.then_some(t))?,
                ))
            }) {
                let sprites = {
                    let sprites = world
                        .em
                        .entities
                        .keys()
                        .cloned()
                        .filter_map(|e| {
                            Some((
                                world
                                    .cm
                                    .get::<Instance>(e, &world.em)
                                    .and_then(|i| i.active.then_some(i))?,
                                world
                                    .cm
                                    .get::<Transform>(e, &world.em)
                                    .and_then(|t| t.active.then_some(t))?,
                            ))
                        })
                        .fold(BTreeMap::<_, Vec<_>>::new(), |mut sprites, (i, t)| {
                            sprites
                                .entry(i.get())
                                .or_insert(Vec::new())
                                .push((i.clone(), t.clone()));

                            sprites
                        });

                    let mut sprites: Vec<_> = sprites
                        .into_values()
                        .filter_map(|mut i| {
                            i.sort_by(|(i1, _), (i2, _)| i1.z.total_cmp(&i2.z));

                            let mut instance_data: Vec<_> = i
                                .iter()
                                .map(|(s, t)| {
                                    let color = s.color.into();
                                    let transform = t.matrix().into();

                                    InstanceData {
                                        z: s.z,
                                        color,
                                        transform,
                                    }
                                })
                                .collect();

                            instance_data.sort_by(|i1, i2| i1.z.total_cmp(&i2.z));

                            Some((i.first().map(|(i, _)| i.clone())?, instance_data))
                        })
                        .collect();

                    sprites.sort_by(|(i1, _), (i2, _)| i1.z.total_cmp(&i2.z));

                    sprites
                };

                let camera_view: [[f32; 4]; 4] = c.view().into();
                let camera_transform: [[f32; 3]; 3] = ct.matrix().into();

                for (s, i) in sprites {
                    let instance_buffer = VertexBuffer::dynamic(&world.display, &i)?;
                    let uniform = uniform! {
                        camera_transform: camera_transform,
                        camera_view: camera_view,
                        tex: Sampler(&*s.texture.buffer, s.texture.sampler_behaviour),
                    };

                    target.draw(
                        (
                            &*s.shape.vertices,
                            instance_buffer
                                .per_instance()
                                .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?,
                        ),
                        NoIndices(s.shape.format),
                        &self.shader.program,
                        &uniform,
                        &s.draw_parameters,
                    )?;
                }
            }
        }

        Ok(())
    }
}
