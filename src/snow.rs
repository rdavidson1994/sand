use crate::element::{Element, ElementId, ElementSetup, GRAVITY, NO_FLAGS};
use crate::fire::FIRE;
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::water::WATER;
use crate::world::World;
use rand::Rng;

pub static SNOW: Element = Element {
    flags: GRAVITY,
    color: [0.9, 0.9, 1.0, 1.0],
    mass: 10,
    id: 11,
    periodic_reaction: Some(|mut snow, _neighbors| {
        if rand::thread_rng().gen_range(0, 100) == 0 {
            snow.edit_state(SNOW.id(), snow.special_info().saturating_add(1));
            // Increase "temperature" by one
            if snow.special_info() == 255 {
                // If we hit 255, melt
                snow.set_element(WATER.id())
            }
        }
        Some(snow)
    }),
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
