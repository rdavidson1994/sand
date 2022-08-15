use crate::element::{
    Element, PeriodicReaction, FLUID, GRAVITY, PAUSE_EXEMPT, PERFECT_RESTITUTION,
};
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::snow::SNOW;
use rand::Rng;

pub static WATER: Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT | FLUID,
    color: [0.0, 0.0, 1.0, 1.0],
    mass: 8,
    id: 6,
    periodic_reaction: PeriodicReaction::Some(|mut this, _world| {
        if this.temperature > 100 {
            // Over designated boiling point, become steam
            this.set_element(STEAM.id())
        } else if this.temperature < 0 {
            this.set_element(SNOW.id())
        } else {
            // Water "jiggles" slightly
            this.velocity.x += rand::thread_rng().gen_range(-3, 3 + 1);
        }
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};

pub static STEAM: Element = Element {
    flags: PAUSE_EXEMPT | PERFECT_RESTITUTION | FLUID,
    color: [0.8, 0.8, 1.0, 1.0],
    mass: 8,
    id: 16,
    periodic_reaction: PeriodicReaction::Some(|mut this, _world| {
        if this.temperature < 99 {
            this.set_element(WATER.id())
        } else {
            // Jiggle, to keep boiling water from staying still
            if this.velocity.x.abs() < 2 && this.velocity.y.abs() < 2 {
                this.velocity.x += rand::thread_rng().gen_range(-3, 3 + 1);
                this.velocity.y += rand::thread_rng().gen_range(-3, 3 + 1);
            }
        }
        Some(this)
    }),
    default_temperature: 120,
    ..ELEMENT_DEFAULT
};
