use crate::element::{PeriodicReaction, GRAVITY, PAUSE_EXEMPT};
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
    periodic_reaction: PeriodicReaction::Some(|mut this, mut world| {
        for i in world.neighbors() {
            match world[i] {
                Some(_) => {
                    // Don't do anything to existing tiles
                }
                None => {
                    // Sprinkle fire into empty ones
                    if rand::thread_rng().gen_range(0, 150) == 0 {
                        world[i] = Some(Tile::new(
                            ElementState::new(FIRE.id(), NO_ASH),
                            Vector {
                                x: rand::thread_rng().gen_range(-126, 127),
                                y: rand::thread_rng().gen_range(-126, 127),
                            },
                            Vector {
                                x: rand::thread_rng().gen_range(
                                    this.velocity.x.saturating_sub(10),
                                    this.velocity.x.saturating_add(10),
                                ),
                                y: rand::thread_rng().gen_range(
                                    this.velocity.y.saturating_sub(10),
                                    this.velocity.y.saturating_add(10),
                                ),
                            },
                        ));
                        // Every time you create fire, roll to cool into rock
                        if rand::thread_rng().gen_range(0, 30) == 0 {
                            this.set_element(ROCK.id());
                        }
                    }
                }
            }
        }
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};

pub struct LavaSetup;
impl ElementSetup for LavaSetup {
    fn register_reactions(&self, world: &mut World) {
        // Lava melts metal
        world.register_collision_reaction(&METAL, &LAVA, |mut metal, lava| {
            metal.set_element(LAVA.id());
            (Some(metal), Some(lava))
        })
    }

    fn build_element(&self) -> Element {
        LAVA.clone()
    }

    fn get_id(&self) -> ElementId {
        LAVA.id()
    }
}
