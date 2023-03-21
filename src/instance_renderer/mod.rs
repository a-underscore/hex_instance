use crate::{Instance, InstanceData};
use hex::{
    anyhow,
    assets::{Shader, Shape},
    components::{Camera, Transform},
    ecs::{ev::Control, system_manager::System, Ev, World},
    glium::{
        draw_parameters::{Blend, DepthTest},
        glutin::event::Event,
        index::NoIndices,
        uniform,
        uniforms::Sampler,
        Depth, Display, DrawParameters, Surface, VertexBuffer,
    },
};
use std::{collections::BTreeMap, rc::Rc};

pub const INSTANCE_VERTEX_SRC: &str = include_str!("instance_vertex.glsl");
pub const INSTANCE_FRAGMENT_SRC: &str = include_str!("instance_fragment.glsl");

pub struct InstanceRenderer<'a> {
    pub draw_parameters: DrawParameters<'a>,
    pub shader: Shader,
    pub shape: Shape,
}

impl InstanceRenderer<'_> {
    pub fn new(display: &Display, shape: Shape) -> anyhow::Result<Self> {
        Ok(Self {
            shader: Shader::new(display, INSTANCE_VERTEX_SRC, INSTANCE_FRAGMENT_SRC, None)?,
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
        })
    }
}

impl<'a, 'b> System<'a> for InstanceRenderer<'b>
where
    'b: 'a,
{
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
                        .fold(BTreeMap::<_, (_, Vec<_>)>::new(), |mut sprites, (i, t)| {
                            let (_, instances) = sprites
                                .entry(Rc::as_ptr(&i.texture))
                                .or_insert((i.texture.clone(), Vec::new()));

                            instances.push((i.clone(), t.clone()));

                            sprites
                        });

                    let sprites: Vec<_> = sprites
                        .into_values()
                        .filter_map(|(t, i)| {
                            let mut instance_data: Vec<_> = i
                                .into_iter()
                                .map(|(s, t)| InstanceData {
                                    z: s.z,
                                    color: s.color,
                                    transform: t.matrix().0,
                                })
                                .collect();

                            instance_data.sort_by(|i1, i2| i1.z.total_cmp(&i2.z));

                            Some((t, instance_data))
                        })
                        .collect();

                    sprites
                };

                for (t, i) in sprites {
                    let instance_buffer = VertexBuffer::dynamic(&world.display, &i)?;
                    let uniform = uniform! {
                        camera_transform: ct.matrix().0,
                        camera_view: c.view().0,
                        tex: Sampler(&*t.buffer, t.sampler_behaviour),
                    };

                    target.draw(
                        (
                            &*self.shape.vertices,
                            instance_buffer
                                .per_instance()
                                .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?,
                        ),
                        NoIndices(self.shape.format),
                        &self.shader.program,
                        &uniform,
                        &self.draw_parameters,
                    )?;
                }
            }
        }

        Ok(())
    }
}
