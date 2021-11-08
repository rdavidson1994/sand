use crate::element::{Element, ElementId, ElementSetup, FIXED, FLUID, GRAVITY, PeriodicReaction};
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::world::World;

pub static GLUE: Element = Element {
    flags: GRAVITY | FLUID,
    color: [0.9, 0.9, 0.5, 1.0],
    mass: 10,
    id: 15,
    ..ELEMENT_DEFAULT
};

pub struct GlueSetup;
impl ElementSetup for GlueSetup {
    fn register_reactions(&self, world: &mut World) {
        world.register_flag_collision_reaction(&GLUE, FIXED, |mut glue_tile, fixed_tile| {
            glue_tile.velocity.x = 0;
            glue_tile.velocity.y = 0;
            glue_tile.set_element(SOLID_GLUE.id());
            (Some(glue_tile), Some(fixed_tile))
        })
    }

    fn build_element(&self) -> Element {
        GLUE.clone()
    }

    fn get_id(&self) -> ElementId {
        GLUE.id()
    }
}

pub static SOLID_GLUE : Element = Element {
    flags: FIXED,
    color: [0.8, 0.8, 0.7, 1.0],
    mass: 10,
    id: 14,
    periodic_reaction: PeriodicReaction::DecayInto {
        element_id: GLUE.id(),
        lifetime: 10,
        rarity: 100
    },
    ..ELEMENT_DEFAULT
};

