use crate::components::Instance;
use hex::{
    anyhow,
    assets::Shape,
    components::{Camera, Trans},
    renderer_manager::{Draw, Renderer},
    ComponentManager, Context, EntityManager,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct InstanceRenderer {
    pub shape: Shape,
}

impl InstanceRenderer {
    pub fn new(shape: Shape) -> Self {
        Self { shape }
    }
}

impl Renderer for InstanceRenderer {
    fn draw(
        &mut self,
        draw: &mut Draw,
        context: Arc<RwLock<Context>>,
        em: Arc<RwLock<EntityManager>>,
        cm: Arc<RwLock<ComponentManager>>,
    ) -> anyhow::Result<()> {
        let context = context.read().unwrap();
        let em = em.read().unwrap();
        let cm = cm.read().unwrap();

        if let Some((ce, c, ct)) = em.entities().keys().cloned().find_map(|e| {
            Some((
                e,
                cm.get::<Camera>(e)
                    .and_then(|c| c.read().unwrap().active.then_some(c))?,
                cm.get::<Trans>(e)
                    .and_then(|t| t.read().unwrap().active.then_some(t))?,
            ))
        }) {
            let instances = {
                let instances = em
                    .entities()
                    .keys()
                    .cloned()
                    .filter_map(|e| {
                        Some((
                            e,
                            cm.get::<Trans>(e)
                                .and_then(|t| t.read().unwrap().active.then_some(t))?,
                            cm.get::<Instance>(e)
                                .and_then(|i| i.read().unwrap().active.then_some(i))?,
                        ))
                    })
                    .fold(
                        HashMap::<_, (_, Vec<_>)>::new(),
                        |mut instances_map, (e, t, i)| {
                            let (_, instances) = {
                                let i = i.read().unwrap();

                                instances_map
                                    .entry((
                                        Arc::as_ptr(&i.texture),
                                        Arc::as_ptr(&i.pipeline),
                                        Arc::as_ptr(&i.drawable),
                                        Arc::as_ptr(&i.shaders),
                                        i.layer,
                                    ))
                                    .or_insert((i.layer, Vec::new()))
                            };

                            instances.push((e, t.clone(), i.clone()));

                            instances_map
                        },
                    );

                let mut instances: Vec<_> = instances
                    .into_values()
                    .filter_map(|(layer, i)| {
                        if !i.is_empty() {
                            let instance = i[0].clone();

                            Some((layer, instance, i))
                        } else {
                            None
                        }
                    })
                    .collect();

                instances.sort_by(|(l1, _, _), (l2, _, _)| l1.cmp(l2));

                instances
            };

            for (_, (_, _, i), instances) in instances {
                let (z, d) = {
                    let i = i.read().unwrap();

                    (c.read().unwrap().calculate_z(i.layer), i.drawable.clone())
                };

                d.draw(
                    (z, self.shape.clone(), instances),
                    (ce, ct.clone(), c.clone()),
                    &context,
                    draw,
                    &em,
                    &cm,
                )?;
            }
        }

        Ok(())
    }
}
