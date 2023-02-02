use crate::instance_id::InstanceId;
use hex::{
    anyhow,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct InstanceRenderer;

impl<'a> System<'a> for InstanceRenderer {
    fn update(&mut self, event: &mut Ev, world: &mut World) -> anyhow::Result<()> {
        if let Ev::Draw((Event::MainEventsCleared, target)) = event {
            let mut ids = HashMap::new();

            for e in world.em.entities.keys().cloned() {
                if let Some(id) = world.cm.get::<InstanceId>(e, &world.em).map(|id| id.get()) {
                    ids.entry(id).or_insert(Vec::new()).push(id);
                }
            }

            for _ in ids.keys().cloned() {
            }
        }
        Ok(())
    }
}
