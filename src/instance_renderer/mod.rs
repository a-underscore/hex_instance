use crate::{Instance, InstanceData};
use hex::{
    anyhow,
    assets::{Shader, Shape},
    components::{Camera, Transform},
    ecs::{system_manager::System, ComponentManager, EntityManager, Ev, Scene},
    glium::{
        draw_parameters::{Blend, DepthTest},
        index::NoIndices,
        uniform,
        uniforms::Sampler,
        Depth, Display, DrawParameters, Surface, VertexBuffer,
    },
};
use std::{collections::BTreeMap, rc::Rc};

pub struct InstanceRenderer<'a> {
    pub draw_parameters: DrawParameters<'a>,
    pub shader: Shader,
    pub shape: Shape,
}

impl InstanceRenderer<'_> {
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

impl<'a, 'b> System<'a> for InstanceRenderer<'b>
where
    'b: 'a,
{
    fn update(
        &mut self,
        event: &mut Ev,
        scene: &mut Scene,
        (em, cm): (&mut EntityManager, &mut ComponentManager),
    ) -> anyhow::Result<()> {
        if let Ev::Draw((_, target)) = event {
            if let Some((c, ct)) = em.entities.keys().cloned().find_map(|e| {
                Some((
                    cm.get::<Camera>(e, em)
                        .and_then(|c| c.active.then_some(c))?,
                    cm.get::<Transform>(e, em)
                        .and_then(|t| t.active.then_some(t))?,
                ))
            }) {
                let sprites = {
                    let sprites = em
                        .entities
                        .keys()
                        .cloned()
                        .filter_map(|e| {
                            Some((
                                cm.get::<Instance>(e, em)
                                    .and_then(|i| i.active.then_some(i))?,
                                cm.get::<Transform>(e, em)
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

                    let mut sprites: Vec<_> = sprites
                        .into_values()
                        .filter_map(|(t, i)| {
                            let instance_data: Vec<_> = i
                                .into_iter()
                                .map(|(s, t)| InstanceData {
                                    z: s.z,
                                    color: s.color,
                                    transform: t.matrix().0,
                                })
                                .collect();

                            Some((
                                t,
                                instance_data
                                    .iter()
                                    .min_by(|i1, i2| i1.z.total_cmp(&i2.z))
                                    .map(|i| i.z)?,
                                instance_data,
                            ))
                        })
                        .collect();

                    sprites.sort_by(|(_, z1, _), (_, z2, _)| z1.total_cmp(z2));

                    sprites
                };

                for (t, _, i) in sprites {
                    let instance_buffer = VertexBuffer::dynamic(&scene.display, &i)?;
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
