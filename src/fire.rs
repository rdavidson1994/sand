use crate::element::{Element, ElementId, ElementSetup, GRAVITY, NO_FLAGS};
use crate::neighbors;
use crate::simple_elements::{ELEMENT_DEFAULT, SAND};
use crate::tile::{ElementState, Tile, Vector};
use crate::water::WATER;
use crate::world::World;
use rand::{thread_rng, Rng};

#[allow(dead_code)]
pub const NO_ASH: u8 = 1;
pub const MAKES_ASH: u8 = 2;

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
    periodic_reaction: Some(|w, i| {
        let mut rng = thread_rng();
        for j in neighbors(i) {
            if let Some(tile) = &mut w[j] {
                if tile.element_id() == SAND.id {
                    tile.edit_state(FIRE.id(), MAKES_ASH);
                }
            }
        }
        if rng.gen_range(0, 100) == 0 {
            let mut made_ash = false;
            if rng.gen_range(0, 3) == 0 {
                if let Some(tile) = &mut w[i] {
                    if tile.special_info() == MAKES_ASH {
                        tile.set_element(ASH.id());
                        made_ash = true;
                    }
                }
            }
            if !made_ash {
                w[i] = None;
            }
        }
    }),
    ..ELEMENT_DEFAULT
};

pub struct FireElementSetup;
impl ElementSetup for FireElementSetup {
    fn register_reactions(&self, world: &mut World) {
        // Fire burns sand
        world.register_collision_side_effect(&SAND, &FIRE, |world, i_other, _i_fire| {
            let mut rng = thread_rng();
            let (other, mut neighbors) = world.mutate_neighbors(i_other);
            other.set_element(FIRE.id());
            neighbors.for_each(|square| {
                if square.is_none() {
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
                    ));
                }
            });
        });

        // // Water extinguishes fire
        // world.register_collision_reaction(&FIRE, &WATER, |_water_tile, fire_tile| {
        //     println!("{}", fire_tile.get_element().id);
        //     fire_tile.set_element(ASH.id());
        // });
        // Water extinguishes fire
        world.register_collision_side_effect(&FIRE, &WATER, |world, i_fire, _i_water| {
            let mut make_ash = false;
            if let Some(fire) = &mut world[i_fire] {
                if fire.special_info() == MAKES_ASH {
                    fire.set_element(ASH.id());
                    make_ash = true;
                }
            }
            if !make_ash {
                world[i_fire] = None;
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
