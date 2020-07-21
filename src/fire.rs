use crate::{
    GRAVITY,
    NO_FLAGS,
    SAND,
    WATER,
    for_neighbors,
    Element,
    World,
    Tile,
    Vector,
    ElementSetup,
    ElementState,
};
use rand::{thread_rng, Rng};

static ASH : Element = Element {
    flags: GRAVITY,
    color: [0.3, 0.3, 0.3, 1.0],
    mass: 3,
    id: 5,
    decay_reaction: None,
};

pub static FIRE : Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.0, 0.0, 1.0],
    mass: 3,
    id: 4,
    decay_reaction: Some(|w, i| {
        let mut rng = thread_rng();
        if *w[i].as_ref().unwrap().get_state() == ElementState::ActiveFire {
            for_neighbors(i, |j| {
                let did_burn = if let Some(tile) = &mut w[j] {
                    if tile.element_id() == SAND.id {
                        tile.set_element(&FIRE);
                        tile.edit_state(ElementState::ActiveFire);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if did_burn {
                    w.unpause(j);
                }
            });
        }
        else {
            w[i].as_mut().unwrap().edit_state(ElementState::ActiveFire);
        }
        if rng.gen_range(0,20) == 0 {
            w.unpause(i);
            if rng.gen_range(0,3) == 0 {
                w[i].as_mut().unwrap().set_element(&ASH);
            }
            else {
                w[i] = None;
            }
        }
    }),
};

pub struct FireElementSetup;
impl ElementSetup for FireElementSetup {
    fn register_reactions(&mut self, world: &mut World) {
        // Fire burns sand
        world.register_collision_reaction(&FIRE, &SAND, |_fire_tile, sand_tile| {
            sand_tile.set_element(&FIRE);
        });
        world.register_collision_side_effect(&FIRE, &SAND, burn);

        // Water extinguishes fire
        world.register_collision_reaction(&FIRE, &WATER, |fire_tile, _water_tile| {
            fire_tile.set_element(&ASH);
        });
    }
}

fn burn(world: &mut World, _fire_loc: usize, other_loc: usize) {
    let mut rng = thread_rng();
    for_neighbors(other_loc, |position| {
        match &world[position] {
            Some(_) => {
                 world[position].as_mut().unwrap().paused =false;
            },
            None => {
                world[position] = Some(Tile::new(
                    &FIRE,
                    ElementState::None,
                    Vector {
                        x: rng.gen_range(-128,127),
                        y: rng.gen_range(-128,127),
                    },
                    Vector {
                        x: rng.gen_range(-10,10),
                        y: rng.gen_range(-10,10),
                    },
                    false
                ));
            }
        }
    });
}

