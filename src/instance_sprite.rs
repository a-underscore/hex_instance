use hex::{cid, components::Sprite, ecs::component_manager::Component};

#[derive(Clone)]
pub struct InstanceSprite<'a>(pub Sprite<'a>);

impl<'a> InstanceSprite<'a> {
    pub fn sprite(&self) -> &Sprite<'a> {
        &self.0
    }
}

impl Component for InstanceSprite<'_> {
    fn id() -> usize {
        cid!()
    }
}
