use crate::instance::Instance;
use hex::{
    anyhow,
    components::Sprite,
    ecs::{
        system_manager::{Ev, System},
        world::World,
    },
    glium::glutin::event::Event,
};

pub struct InstanceRenderer<'a> {
    pub instanced_objects: World<'a>,
}

impl<'a, 'b> System<'a> for InstanceRenderer<'b>
where
    'b: 'a,
{
    fn update(&mut self, event: &mut Ev, world: &mut World<'a>) -> anyhow::Result<()> {
        if let Ev::Draw((Event::MainEventsCleared, target)) = event {
            for e in world.em.entities.keys().cloned() {
                if let (Some(id), Some(s)) = (
                    world.cm.get::<Instance>(e, &world.em),
                    world.cm.get::<Sprite>(e, &world.em),
                ) {}
            }
        }

        Ok(())
    }
}
