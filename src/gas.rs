use crate::element::{Element, PAUSE_EXEMPT, PERFECT_RESTITUTION};
use crate::fire::FIRE;
use crate::neighbors;
use crate::simple_elements::ELEMENT_DEFAULT;
use crate::tile::{ElementState, Tile};
use crate::Vector;

const EXPLOSION_VELOCITY: i8 = 50;

const ADJ_VEL: i8 = EXPLOSION_VELOCITY;
const DIAG_VEL: i8 = ((EXPLOSION_VELOCITY as f64) * 1.414 / 2.0) as i8; // i.e. times sqrt(2)/2

#[rustfmt::skip]
const EXPLOSION_VECTORS: [(i8, i8); 8] = [
    (-DIAG_VEL, -DIAG_VEL), (0, -ADJ_VEL), (DIAG_VEL, -DIAG_VEL),
    (-ADJ_VEL, 0),          /*No center*/  (ADJ_VEL, 0),
    (-DIAG_VEL, DIAG_VEL),  (0, ADJ_VEL),  (DIAG_VEL, DIAG_VEL),
];

pub static GAS: Element = Element {
    flags: PAUSE_EXEMPT | PERFECT_RESTITUTION,
    color: [1.0, 0.5, 1.0, 1.0],
    mass: 3,
    id: 3,
    periodic_reaction: Some(|world, i| {
        let mut will_explode = false;
        for j in neighbors(i) {
            if let Some(tile) = &mut world[j] {
                if tile.element_id() == FIRE.id {
                    will_explode = true;
                    break;
                }
            }
        }
        if will_explode {
            for (j, delta_v) in neighbors(i).zip(EXPLOSION_VECTORS.iter()) {
                let mut new_tile = match world[j].take() {
                    Some(existing_tile) => existing_tile,
                    None => Tile::new(
                        ElementState::default(FIRE.id()),
                        Vector { x: 0, y: 0 },
                        Vector { x: 0, y: 0 },
                        false,
                    ),
                };
                new_tile.velocity.x = new_tile.velocity.x.saturating_add(delta_v.0);
                new_tile.velocity.y = new_tile.velocity.x.saturating_add(delta_v.1);
                world[j] = Some(new_tile);
            }
            if let Some(this_tile) = &mut world[i] {
                this_tile.set_element(FIRE.id());
            }
        }
    }),
    ..ELEMENT_DEFAULT
};
