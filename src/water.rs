use crate::simple_elements::ELEMENT_DEFAULT;
use crate::{Element, GRAVITY, PAUSE_EXEMPT};
use rand::Rng;

pub static WATER: Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT,
    color: [0.0, 0.0, 1.0, 1.0],
    mass: 8,
    id: 6,
    periodic_side_effect: Some(|world, position| {
        // Water "jiggles" slightly
        let water_tile = world[position].as_mut().unwrap();
        water_tile.velocity.x += rand::thread_rng().gen_range(-3, 3 + 1);
    }),
    ..ELEMENT_DEFAULT
};
