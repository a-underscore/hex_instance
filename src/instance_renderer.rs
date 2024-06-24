use crate::components::Instance;
use hex::{
    anyhow,
    components::{Camera, Trans},
    renderer_manager::{Draw, Renderer},
    ComponentManager, Context, EntityManager,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub struct InstanceRenderer;

impl Renderer for InstanceRenderer {
    fn draw(
        &mut self,
        draw: &mut Draw,
        context: Arc<RwLock<Context>>,
        em: Arc<RwLock<EntityManager>>,
        cm: Arc<RwLock<ComponentManager>>,
    ) -> anyhow::Result<()> {
        if let Some(((ce, c, ct), instances)) = {
            let em = em.read().unwrap();
            let cm = cm.read().unwrap();

            if let Some(c) = em
                .entities()
                .find_map(|e| Some((e, cm.get::<Camera>(e)?.clone(), cm.get::<Trans>(e)?.clone())))
            {
                let instances = em
                    .entities()
                    .filter_map(|e| {
                        Some((
                            e,
                            cm.get::<Trans>(e)?.clone(),
                            cm.get::<Instance>(e)?.clone(),
                        ))
                    })
                    .fold(
                        HashMap::<_, (_, Vec<_>)>::new(),
                        |mut instances_map, (e, t, i)| {
                            let (_, instances) = {
                                let i = i.read().unwrap();

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

                            instances.push((e, t.clone(), i.clone()));

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

                instances.sort_by(|(l1, _, _), (l2, _, _)| l1.cmp(l2));

                Some((c, instances))
            } else {
                None
            }
        } {
            for (_, (_, _, i), instances) in instances {
                let d = i.read().unwrap().drawable.clone();

                d.write().unwrap().draw(
                    instances,
                    (ce, ct.clone(), c.clone()),
                    draw,
                    context.clone(),
                    em.clone(),
                    cm.clone(),
                )?;
            }
        }

        Ok(())
    }
}
