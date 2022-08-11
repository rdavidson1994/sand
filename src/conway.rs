use crate::element::{Element, PeriodicReaction, FIXED};
use crate::metal::{CHARGED_HEAD, METAL, NEUTRAL};
use crate::simple_elements::ELEMENT_DEFAULT;

const DEAD: u8 = 1;
const ALIVE: u8 = 2;

#[allow(dead_code)]
pub static CONWAY: Element = Element {
    flags: FIXED,
    color: [0.0, 0.0, 0.0, 1.0],
    mass: 10,
    id: 13,
    state_colors: Some(|state| {
        if state == ALIVE {
            &[0.0, 1.0, 0.0, 1.0]
        } else {
            &[0.3, 0.5, 0.3, 1.0]
        }
    }),
    periodic_reaction: PeriodicReaction::Some(|mut this, mut world| {
        let mut alive_neighbors = 0;

        for i in world.neighbors() {
            if let Some(tile) = &mut world[i] {
                if tile.is_charged_metal() {
                    // If you meet an electron, become alive
                    this.edit_state(CONWAY.id(), ALIVE);
                    // Early return yourself
                    return Some(this);
                }
                if tile.has_state(CONWAY.id(), ALIVE) {
                    alive_neighbors += 1;
                }
            }
        }
        if alive_neighbors == 3 {
            this.edit_state(CONWAY.id(), ALIVE);
        } else if this.special_info() == ALIVE {
            if alive_neighbors > 3 {
                // Death by overcrowding
                this.edit_state(CONWAY.id(), DEAD);
                for i in world.neighbors() {
                    if let Some(tile) = &mut world[i] {
                        if tile.has_state(METAL.id(), NEUTRAL) {
                            tile.edit_state(METAL.id(), CHARGED_HEAD)
                        }
                    }
                }
            } else if alive_neighbors < 2 {
                // Death by loneliness
                this.edit_state(CONWAY.id(), DEAD);
            }
        }
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};
