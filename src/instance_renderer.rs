use crate::{Instance, InstanceData, InstanceSprite, INSTANCE_FRAGMENT_SRC, INSTANCE_VERTEX_SRC};
use hex::{
    anyhow,
    assets::Shader,
    components::{Camera, Transform},
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::{
        index::NoIndices, uniform, uniforms::Sampler, Display, Program, Surface, VertexBuffer,
    },
};
use std::{collections::BTreeMap, rc::Rc};

pub struct InstanceRenderer {
    pub shader: Shader,
}

impl InstanceRenderer {
    pub fn new(display: &Display) -> anyhow::Result<Self> {
        Ok(Self {
            shader: Shader {
                program: Rc::new(Program::from_source(
                    display,
                    INSTANCE_VERTEX_SRC,
                    INSTANCE_FRAGMENT_SRC,
                    None,
                )?),
            },
        })
    }
}

impl<'a> System<'a> for InstanceRenderer {
    fn update(&mut self, event: &mut Ev, world: &mut World<'a>) -> anyhow::Result<()> {
        if let Ev::Draw((_, target)) = event {
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
                    let mut sprites: BTreeMap<_, Vec<_>> = BTreeMap::new();

                    for e in world.em.entities.keys().cloned() {
                        if let Some((i, s, t)) =
                            world.cm.get::<Instance>(e, &world.em).and_then(|i| {
                                Some((
                                    i,
                                    world
                                        .cm
                                        .get::<InstanceSprite>(e, &world.em)
                                        .and_then(|s| s.sprite().active.then_some(s))?,
                                    world
                                        .cm
                                        .get::<Transform>(e, &world.em)
                                        .and_then(|t| t.active.then_some(t))?,
                                ))
                            })
                        {
                            sprites.entry(i.get()).or_default().push((s, t));
                        }
                    }

                    let mut sprites: Vec<_> = sprites.into_values().collect();

                    sprites.sort_by(|s1, s2| s1[0].0.sprite().z.total_cmp(&s2[0].0.sprite().z));

                    sprites
                };

                let camera_view: [[f32; 4]; 4] = c.view().into();
                let camera_transform: [[f32; 3]; 3] = ct.matrix().into();

                for i in sprites {
                    let (s, _) = i[0];
                    let instance_data: Vec<_> = i
                        .iter()
                        .map(|(s, t)| {
                            let color = s.sprite().color.into();
                            let transform = t.matrix().into();

                            InstanceData {
                                z: s.sprite().z,
                                color,
                                transform,
                            }
                        })
                        .collect();
                    let instance_buffer = VertexBuffer::dynamic(&world.display, &instance_data)?;
                    let uniform = uniform! {
                        camera_transform: camera_transform,
                        camera_view: camera_view,
                        image: Sampler(&*s.sprite().texture.buffer, s.sprite().texture.sampler_behaviour),
                    };

                    target.draw(
                        (
                            &*s.sprite().shape.vertices,
                            instance_buffer.per_instance().unwrap(),
                        ),
                        NoIndices(s.sprite().shape.format),
                        &self.shader.program,
                        &uniform,
                        &s.sprite().draw_parameters,
                    )?;
                }
            }
        }

        Ok(())
    }
}
