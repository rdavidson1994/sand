use crate::{
    dirt::{dirt_moisture, DIRT},
    element::{Element, PeriodicReaction, FIXED, GRAVITY},
    simple_elements::ELEMENT_DEFAULT,
    tile::{ElementState, Tile},
};

pub static SEED: Element = Element {
    flags: GRAVITY,
    color: [0.5, 0.6, 0.1, 1.0],
    mass: 10,
    id: 19,
    periodic_reaction: PeriodicReaction::Some(|this, mut world| {
        let mut should_grow = false;
        let mut total_moisture: u8 = 0;
        if this.velocity.is_zero() {
            return Some(this);
        }
        let dirt_or_empty_above = world
            .above()
            .as_ref()
            .map_or(true, |x| x.element_id() == DIRT.id);
        if dirt_or_empty_above {
            world.for_each_neighbor(|neighbor| {
                let moisture = neighbor.as_ref().map_or(0, |x| dirt_moisture(&x));
                if moisture > 64 {
                    if let Some(neighbor) = neighbor {
                        total_moisture = total_moisture.saturating_add(10);
                        neighbor.adjust_info(-10);
                        should_grow = true;
                    }
                }
            });
            if should_grow && total_moisture > 20 {
                *world.above() = Some(Tile::stationary(
                    ElementState::new(PLANT.id(), total_moisture),
                    this.temperature,
                ))
            }
        }

        Some(this)
    }),
    ..ELEMENT_DEFAULT
};

pub static PLANT: Element = Element {
    flags: FIXED,
    color: [0.1, 0.8, 0.1, 1.0],
    mass: 3,
    id: 20,
    periodic_reaction: PeriodicReaction::Some(|mut this, mut world| {
        if let Some(below) = world.below() {
            if below.special_info() > 10
                && this.special_info() < 192
                && below.element_id() == PLANT.id
            {
                this.adjust_info(10);
                below.adjust_info(-10);
            }
        }

        if this.special_info() > 20 {
            let above = world.above();
            let dirt_or_empty_above = above.as_ref().map_or(true, |x| x.element_id() == DIRT.id);
            if dirt_or_empty_above {
                *above = Some(Tile::stationary(
                    ElementState::new(PLANT.id(), 1),
                    this.temperature,
                ));
                this.adjust_info(-10)
            }
        }

        Some(this)
    }),
    ..ELEMENT_DEFAULT
};

pub static ROOT: Element = Element {
    flags: FIXED,
    color: [0.3, 0.3, 0.1, 1.0],
    mass: 10,
    id: 21,
    periodic_reaction: PeriodicReaction::Some(|this, mut _world| Some(this)),
    ..ELEMENT_DEFAULT
};
