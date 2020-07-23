use crate::{
    Element,
    ElementSetup,
    ElementId,
    GRAVITY,
    World,
};

pub static SAND : Element = Element {
    flags: GRAVITY,
    color: [1.0, 1.0, 0.5, 1.0],
    mass: 10,
    id: 2,
    decay_reaction: None,
};

pub struct SandSetup;
impl ElementSetup for SandSetup {
    fn register_reactions(&self, _world: &mut World) { }
    fn build_element(&self) -> Element {
        SAND.clone()
    }
    fn get_id(&self) -> ElementId { 
        SAND.id()
    }
}