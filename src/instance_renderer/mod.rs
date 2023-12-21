use crate::{Instance, InstanceData};
use hex::{
    anyhow,
    assets::{Shader, Shape},
    components::{Camera, Transform},
    ecs::{system_manager::System, ComponentManager, Context, EntityManager, Ev},
    glium::{
        draw_parameters::{Blend, DepthTest},
        index::NoIndices,
        uniform,
        uniforms::Sampler,
        Depth, Display, DrawParameters, Surface, VertexBuffer,
    },
};
use ordered_float::OrderedFloat;
use std::{collections::HashMap, rc::Rc};

pub struct InstanceRenderer {
    pub shader: Shader,
    pub draw_parameters: DrawParameters<'static>,
    pub shape: Shape,
}

impl InstanceRenderer {
    pub fn new(display: &Display, shape: Shape) -> anyhow::Result<Self> {
        Ok(Self {
            shader: Shader::new(
                display,
                include_str!("instance_vertex.glsl"),
                include_str!("instance_fragment.glsl"),
                None,
            )?,
            draw_parameters: DrawParameters {
                depth: Depth {
                    test: DepthTest::IfLessOrEqual,
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

impl System for InstanceRenderer {
    fn update(
        &mut self,
        ev: &mut Ev,
        context: &mut Context,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Draw((_, target)) = ev {
            if let Some((c, ct)) = em.entities().find_map(|e| {
                Some((
                    cm.get_ref::<Camera>(e)
                        .and_then(|c| c.active.then_some(c))?,
                    cm.get_ref::<Transform>(e)
                        .and_then(|t| t.active.then_some(t))?,
                ))
            }) {
                let sprites = {
                    let sprites = em
                        .entities()
                        .filter_map(|e| {
                            Some((
                                cm.get_ref::<Instance>(e)
                                    .and_then(|i| i.active.then_some(i))?,
                                cm.get_ref::<Transform>(e)
                                    .and_then(|t| t.active.then_some(t))?,
                            ))
                        })
                        .fold(HashMap::<_, (_, Vec<_>)>::new(), |mut sprites, (i, t)| {
                            let (_, instances) = sprites
                                .entry((Rc::as_ptr(&i.texture), OrderedFloat(i.z)))
                                .or_insert((i.texture.clone(), Vec::new()));

                            instances.push((i.clone(), t.clone()));

                            sprites
                        });

                    let mut sprites: Vec<_> = sprites
                        .into_iter()
                        .map(|((_, z), (t, i))| {
                            let instance_data: Vec<_> = i
                                .into_iter()
                                .map(|(s, t)| InstanceData {
                                    z: s.z,
                                    color: s.color,
                                    transform: t.matrix().0,
                                })
                                .collect();

                            (z, instance_data, t)
                        })
                        .collect();

                    sprites.sort_by_key(|(z, _, _)| *z);

                    sprites
                };

                for (_, i, t) in sprites {
                    let instance_buffer = VertexBuffer::dynamic(&context.display, &i)?;
                    let uniform = uniform! {
                        camera_transform: ct.matrix().0,
                        camera_proj: c.proj().0,
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
