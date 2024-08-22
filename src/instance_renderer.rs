use crate::components::Instance;
use hex::{
    anyhow,
    components::{Camera, Trans},
    parking_lot::RwLock,
    world::renderer_manager::{Draw, Renderer},
    Context, World,
};
use std::{collections::HashMap, sync::Arc};

pub struct InstanceRenderer;

impl Renderer for InstanceRenderer {
    fn draw(
        &mut self,
        draw: &mut Draw,
        context: Arc<RwLock<Context>>,
        world: Arc<RwLock<World>>,
    ) -> anyhow::Result<()> {
        let res = {
            let em = world.read().em.clone();
            let em = em.read();

            em.entities()
                .find_map(|e| {
                    Some((
                        e,
                        em.get_component::<Camera>(e)?,
                        em.get_component::<Trans>(e)?,
                    ))
                })
                .map(|c| {
                    let instances = em
                        .entities()
                        .filter_map(|e| {
                            Some((
                                e,
                                em.get_component::<Instance>(e)?,
                                em.get_component::<Trans>(e)?,
                            ))
                        })
                        .fold(
                            HashMap::new(),
                            |mut instances_map, ref ie @ (_, ref i, _)| {
                                let (_, instances) = {
                                    let i = i.read();

                                    instances_map
                                        .entry((
                                            Arc::as_ptr(&i.shape),
                                            Arc::as_ptr(&i.texture),
                                            Arc::as_ptr(&i.pipeline),
                                            Arc::as_ptr(&i.drawable),
                                            i.layer,
                                        ))
                                        .or_insert((i.layer, Vec::new()))
                                };

                                instances.push(ie.clone());

                                instances_map
                            },
                        );

                    let mut instances: Vec<_> = instances
                        .into_values()
                        .filter_map(|(layer, i)| {
                            let instance = i.first()?.clone();

                            Some((layer, instance, i))
                        })
                        .collect();

                    instances.sort_by_key(|(l, _, _)| *l);

                    (c, instances)
                })
        };

        if let Some(((ce, c, ct), instances)) = res {
            for (_, (_, i, _), instances) in instances {
                let d = i.read().drawable.clone();

                d.draw(
                    instances,
                    (ce, c.clone(), ct.clone()),
                    draw,
                    context.clone(),
                    world.clone(),
                )?;
            }
        }

        Ok(())
    }
}
