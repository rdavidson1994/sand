use crate::element::{Element, ElementId, ElementSetup, PeriodicReaction, GRAVITY, NO_FLAGS};
use crate::simple_elements::{ELEMENT_DEFAULT, SAND};
use crate::tile::{ElementState, Tile, Vector};
use crate::water::{STEAM, WATER};
use crate::world::World;
use rand::{thread_rng, Rng};

#[allow(dead_code)]
pub const BURNS_CLEAN: u8 = 1;
pub const MAKES_ASH: u8 = 2;
pub const MAKES_WATER: u8 = 3;

pub static ASH: Element = Element {
    flags: GRAVITY,
    color: [0.1, 0.1, 0.1, 1.0],
    mass: 3,
    id: 5,
    ..ELEMENT_DEFAULT
};

pub static FIRE: Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.0, 0.0, 1.0],
    mass: 3,
    id: 4,
    periodic_reaction: PeriodicReaction::Some(|mut this, _world| {
        if this.temperature < 300 || thread_rng().gen_range(0, 200) == 0 {
            return {
                if thread_rng().gen_range(0, 3) == 0 {
                    match this.special_info() {
                        MAKES_ASH => this.set_element(ASH.id()),
                        MAKES_WATER => this.set_element(WATER.id()),
                        _ => {}
                    }
                    Some(this)
                } else {
                    None
                }
            };
        }

        Some(this)
    }),
    default_temperature: 500,
    ..ELEMENT_DEFAULT
};

pub struct FireElementSetup;
impl ElementSetup for FireElementSetup {
    fn register_reactions(&self, world: &mut World) {
        // Fire burns sand
        world.register_collision_side_effect(&SAND, &FIRE, |mut sand, fire, mut world| {
            let mut rng = thread_rng();
            sand.temperature += 400;
            sand.edit_state(FIRE.id(), MAKES_ASH);
            world.for_neighbors_of_first(|square| match square {
                Some(_tile) => {}
                None => {
                    *square = Some(Tile::new(
                        ElementState::default(FIRE.id()),
                        Vector {
                            x: rng.gen_range(-126, 127),
                            y: rng.gen_range(-126, 127),
                        },
                        Vector {
                            x: rng.gen_range(-10, 10),
                            y: rng.gen_range(-10, 10),
                        },
                        (sand.temperature + fire.temperature) / 2,
                    ));
                }
            });
            (Some(sand), Some(fire))
        });

        world.register_collision_side_effect(&FIRE, &WATER, |mut fire, water, _world| {
            if fire.special_info() == MAKES_ASH {
                fire.set_element(ASH.id());
                // If this fire tile will make ash,
                // It transforms into ash
                (Some(fire), Some(water))
            } else if fire.special_info() == MAKES_WATER {
                fire.set_element(STEAM.id());
                (Some(fire), Some(water))
            } else {
                // Otherwise it's deleted
                (None, Some(water))
            }
        });
    }

    fn build_element(&self) -> Element {
        FIRE.clone()
    }

    fn get_id(&self) -> ElementId {
        FIRE.id()
    }
}
