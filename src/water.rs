use crate::element::{Element, PeriodicReaction, FLUID, GRAVITY, PAUSE_EXEMPT};
use crate::simple_elements::ELEMENT_DEFAULT;
use rand::Rng;

pub static WATER: Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT | FLUID,
    color: [0.0, 0.0, 1.0, 1.0],
    mass: 8,
    id: 6,
    periodic_reaction: PeriodicReaction::Some(|mut this, _world| {
        // Water "jiggles" slightly
        this.velocity.x += rand::thread_rng().gen_range(-3, 3 + 1);
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};
