use crate::fire::{FIRE, NO_ASH};
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::tile::{ElementState, Tile, Vector};
use crate::{Element, GRAVITY, PAUSE_EXEMPT, ROCK};
use rand::Rng;

pub static LAVA: Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT,
    color: [0.8, 0.5, 0.2, 1.0],
    mass: 50,
    id: 9,
    periodic_side_effect: Some(|world, position| {
        let (this, mut neighbors) = world.mutate_neighborhood(position);
        neighbors.for_each(|tile| match tile {
            Some(_) => {
                // Don't do anything to existing tiles
            }
            None => {
                // Sprinkle fire into empty ones
                if rand::thread_rng().gen_range(0, 150) == 0 {
                    *tile = Some(Tile::new(
                        ElementState::new(FIRE.id(), NO_ASH),
                        Vector {
                            x: rand::thread_rng().gen_range(-126, 127),
                            y: rand::thread_rng().gen_range(-126, 127),
                        },
                        Vector {
                            x: rand::thread_rng().gen_range(-10, 10),
                            y: rand::thread_rng().gen_range(-10, 10),
                        },
                        false,
                    ));
                    // Every time you create fire, roll to cool into rock
                    if rand::thread_rng().gen_range(0, 30) == 0 {
                        this.set_element(ROCK.id());
                    }
                }
            }
        })
    }),
    ..ELEMENT_DEFAULT
};
