use crate::{Instance, InstanceData, INSTANCE_FRAGMENT_SRC, INSTANCE_VERTEX_SRC};
use hex::{
    anyhow,
    assets::Shader,
    components::{Camera, Transform},
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::{
        index::NoIndices,
        texture::{RawImage2d, Texture2dArray},
        uniform,
        uniforms::MagnifySamplerFilter,
        Display, Program, Surface, VertexBuffer,
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
                    let mut sprites: BTreeMap<_, (_, Vec<_>)> = BTreeMap::new();

                    for e in world.em.entities.keys().cloned() {
                        if let Some((i, t)) = world.cm.get::<Instance>(e, &world.em).and_then(|i| {
                            Some((
                                i.active.then_some(i)?,
                                world
                                    .cm
                                    .get::<Transform>(e, &world.em)
                                    .and_then(|t| t.active.then_some(t))?,
                            ))
                        }) {
                            sprites
                                .entry(i.get())
                                .or_insert((i, Default::default()))
                                .1
                                .push((i, t));
                        }
                    }

                    let mut sprites: Vec<_> = sprites.into_iter().collect();

                    sprites.sort_by(|(_, (s1, _)), (_, (s2, _))| s1.z.total_cmp(&s2.z));

                    sprites
                };

                let camera_view: [[f32; 4]; 4] = c.view().into();
                let camera_transform: [[f32; 3]; 3] = ct.matrix().into();
                let (id_map, texture_data): (BTreeMap<_, _>, Vec<_>) = sprites
                    .iter()
                    .enumerate()
                    .map(|(i, (id, (s, _)))| {
                        let t = RawImage2d {
                            data: s.texture.data.clone(),
                            ..*s.texture
                        };

                        ((id, i), t)
                    })
                    .unzip();
                let texture = Texture2dArray::new(&world.display, texture_data)?;

                for (id, (s, i)) in &sprites {
                    let instance_data: Vec<_> = i
                        .iter()
                        .filter_map(|(s, t)| {
                            let color = s.color.into();
                            let transform = t.matrix().into();

                            Some(InstanceData {
                                z: s.z,
                                color,
                                transform,
                                id: *id_map.get(id)? as f32,
                            })
                        })
                        .collect();
                    let instance_buffer = VertexBuffer::dynamic(&world.display, &instance_data)?;
                    let uniform = uniform! {
                        camera_transform: camera_transform,
                        camera_view: camera_view,
                        tex: texture.sampled().magnify_filter(MagnifySamplerFilter::Nearest),
                    };

                    target.draw(
                        (&*s.shape.vertices, instance_buffer.per_instance().unwrap()),
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
