use crate::element::{Element, ElementId, ElementSetup, NO_FLAGS};
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::world::World;
use crate::WALL;

pub static ERASER: Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.0, 1.0, 1.0],
    mass: 0,
    id: 13,
    periodic_reaction: Some(|_this, mut world| { 
        world.for_each_neighbor(|opt_tile| {
            match opt_tile {
                Some(tile) if tile.element_id() == WALL.id => {
                    // Do nothing to walls
                }
                _ => {
                    // Erase anything else
                    *opt_tile = None
                }
            }
        });
        // Immediately self-delete
        None

    }),
    ..ELEMENT_DEFAULT
};

// Sample setup implementation
pub struct EraserSetup;
impl ElementSetup for EraserSetup {
    fn register_reactions(&self, _world: &mut World) {
        // register your reactions here,
        // by calling world.register_collision_reaction
        // or world.register_collision_side_effect
    }

    fn build_element(&self) -> Element {
        ERASER.clone()
    }

    fn get_id(&self) -> ElementId {
        ERASER.id()
    }
}
