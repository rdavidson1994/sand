use crate::{
    for_neighbors, neighbors, Element, ElementId, ElementSetup, ElementState, Tile, Vector, World,
    ELEMENT_DEFAULT, GRAVITY, NO_FLAGS, SAND, WATER,
};
use rand::{thread_rng, Rng};

pub(crate) static ASH: Element = Element {
    flags: GRAVITY,
    color: [0.3, 0.3, 0.3, 1.0],
    mass: 3,
    id: 5,
    ..ELEMENT_DEFAULT
};

pub static FIRE: Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.0, 0.0, 1.0],
    mass: 3,
    id: 4,
    periodic_side_effect: Some(|w, i| {
        let mut rng = thread_rng();
        for j in neighbors(i) {
            let mut did_burn = false;
            if let Some(tile) = &mut w[j] {
                if tile.element_id() == SAND.id {
                    tile.set_element(FIRE.id());
                    did_burn = true;
                }
            }
            if did_burn {
                w.unpause(j);
            }
        }
        if rng.gen_range(0, 20) == 0 {
            w.unpause(i);
            if rng.gen_range(0, 3) == 0 {
                if let Some(tile) = &mut w[i] {
                    tile.set_element(ASH.id())
                }
            } else {
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
        world.register_collision_side_effect(&FIRE, &SAND, |world, i_fire, i_other| {
            let mut rng = thread_rng();
            let (other, mut neighbors) = world.mutate_neighborhood(i_other);
            other.set_element(FIRE.id());
            neighbors.for_each(|square| match square {
                Some(tile) => {
                    tile.paused = false;
                }
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
                        false,
                    ));
                }
            });
        });

        // Water extinguishes fire
        world.register_collision_reaction(&FIRE, &WATER, |fire_tile, _water_tile| {
            fire_tile.set_element(ASH.id());
        });
    }

    fn build_element(&self) -> Element {
        FIRE.clone()
    }

    fn get_id(&self) -> ElementId {
        ElementId(FIRE.id)
    }
}

fn burn(world: &mut World, _fire_loc: usize, other_loc: usize) {
    let mut rng = thread_rng();
    for_neighbors(other_loc, |position| match &world[position] {
        Some(_) => {
            world[position].as_mut().unwrap().paused = false;
        }
        None => {
            world[position] = Some(Tile::new(
                ElementState::default(FIRE.id()),
                Vector {
                    x: rng.gen_range(-126, 127),
                    y: rng.gen_range(-126, 127),
                },
                Vector {
                    x: rng.gen_range(-10, 10),
                    y: rng.gen_range(-10, 10),
                },
                false,
            ));
        }
    });
}
