use crate::element::{Element, ElementId, ElementSetup, NO_FLAGS};
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::world::World;

pub static YOUR_ELEMENT: Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.0, 1.0, 1.0],
    mass: 10,
    id: 0,
    ..ELEMENT_DEFAULT
};

// Sample setup implementation
pub struct YourElementSetup;
impl ElementSetup for YourElementSetup {
    fn register_reactions(&self, world: &mut World) {
        // register your reactions here,
        // by calling world.register_collision_reaction
        // or world.register_collision_side_effect
    }

    fn build_element(&self) -> Element {
        YOUR_ELEMENT.clone()
    }

    fn get_id(&self) -> ElementId {
        YOUR_ELEMENT.id()
    }
}
