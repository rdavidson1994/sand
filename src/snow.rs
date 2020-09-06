use crate::element::{Element, ElementId, ElementSetup, PeriodicReaction, GRAVITY};
use crate::fire::FIRE;
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::water::WATER;
use crate::world::World;

pub static SNOW: Element = Element {
    flags: GRAVITY,
    color: [0.9, 0.9, 1.0, 1.0],
    mass: 10,
    id: 11,
    periodic_reaction: PeriodicReaction::DecayInto {
        element_id: WATER.id(),
        lifetime: 100,
        rarity: 255,
    },
    ..ELEMENT_DEFAULT
};

// Sample setup implementation
pub struct SnowSetup;
impl ElementSetup for SnowSetup {
    fn register_reactions(&self, world: &mut World) {
        // fire melts snow
        world.register_collision_reaction(&FIRE, &SNOW, |fire, mut snow| {
            snow.set_element(WATER.id());
            (Some(fire), Some(snow))
        });
    }

    fn build_element(&self) -> Element {
        SNOW.clone()
    }

    fn get_id(&self) -> ElementId {
        SNOW.id()
    }
}
