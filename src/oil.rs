use crate::element::{Element, GRAVITY};
use crate::simple_elements::ELEMENT_DEFAULT;
use rand::Rng;

pub static OIL: Element = Element {
    flags: GRAVITY,
    color: [0.4, 0.2, 0.1, 1.0],
    mass: 20,
    id: 12,
    periodic_reaction: Some(|this, mut world| {
        if let Some(ref mut tile) = world.above() {
            // If there is a tile above you, it tries to "slide off" randomly
            let delta_x = rand::thread_rng().gen_range(-3, 3 + 1);
            tile.velocity.x = tile.velocity.x.saturating_add(delta_x);
        }
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};
