use crate::element::{GRAVITY, PAUSE_EXEMPT};
use crate::fire::{FIRE, NO_ASH};
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::tile::{ElementState, Tile, Vector};
use crate::world::World;
use crate::{Element, ElementId, ElementSetup, METAL, ROCK};
use rand::Rng;

pub static LAVA: Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT,
    color: [0.8, 0.5, 0.2, 1.0],
    mass: 50,
    id: 9,
    periodic_reaction: Some(|world, position| {
        let (this, mut neighbors) = world.mutate_neighbors(position);
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

pub struct LavaSetup;
impl ElementSetup for LavaSetup {
    fn register_reactions(&self, world: &mut World) {
        // Lava melts metal
        world.register_collision_reaction(&METAL, &LAVA, |metal, _lava| {
            metal.get_element().id;
            metal.set_element(LAVA.id());
        })
    }

    fn build_element(&self) -> Element {
        LAVA.clone()
    }

    fn get_id(&self) -> ElementId {
        LAVA.id()
    }
}
