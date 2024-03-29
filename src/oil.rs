use crate::element::{Element, PeriodicReaction, GRAVITY};
use crate::fire::{BURNS_CLEAN, FIRE};
use crate::simple_elements::ELEMENT_DEFAULT;
use rand::Rng;

const OIL_BURN_TEMPERATURE: i16 = 280;

pub static OIL: Element = Element {
    flags: GRAVITY,
    color: [0.4, 0.2, 0.1, 1.0],
    mass: 20,
    id: 12,
    periodic_reaction: PeriodicReaction::Some(|mut this, mut world| {
        if let Some(ref mut tile) = world.above() {
            // If there is a tile above you, it tries to "slide off" randomly
            let delta_x = rand::thread_rng().gen_range(-3, 3 + 1);
            tile.velocity.x = tile.velocity.x.saturating_add(delta_x);
        }
        if this.temperature > OIL_BURN_TEMPERATURE {
            this.edit_state(FIRE.id(), BURNS_CLEAN);
            this.temperature += 1000;
            if rand::thread_rng().gen_bool(0.5) {
                this.velocity.x =
                    this.velocity
                        .x
                        .saturating_add(if rand::thread_rng().gen_bool(0.5) {
                            -50
                        } else {
                            50
                        });
            } else {
                this.velocity.y =
                    this.velocity
                        .y
                        .saturating_add(if rand::thread_rng().gen_bool(0.5) {
                            -50
                        } else {
                            50
                        });
            }
        }
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};
