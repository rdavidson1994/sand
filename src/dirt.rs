use crate::{
    element::{Color, Element, ElementSetup, PeriodicReaction, GRAVITY},
    simple_elements::ELEMENT_DEFAULT,
    tile::Tile,
    water::WATER,
};

const DRY_COLOR: Color = [0.6, 0.6, 0.2, 1.0];
const WET_64_COLOR: Color = [0.5, 0.5, 0.25, 1.0];
const WET_128_COLOR: Color = [0.4, 0.4, 0.2, 1.0];
const WET_192_COLOR: Color = [0.3, 0.3, 0.15, 1.0];
const WET_255_COLOR: Color = [0.2, 0.2, 0.1, 1.0];

pub static DIRT: Element = Element {
    flags: GRAVITY,
    color: [1.0, 1.0, 0.5, 1.0],
    mass: 10,
    id: 18,
    state_colors: Some(|moisture| {
        if moisture == 0 {
            &DRY_COLOR
        } else if moisture <= 64 {
            &WET_64_COLOR
        } else if moisture <= 128 {
            &WET_128_COLOR
        } else if moisture <= 192 {
            &WET_192_COLOR
        } else {
            &WET_255_COLOR
        }
    }),
    periodic_reaction: PeriodicReaction::Some(|mut this, mut world| {
        world.for_each_neighbor(|neighbor| match neighbor {
            Some(tile) => {
                if dirt_moisture(tile) > dirt_moisture(&this).saturating_add(5) {
                    tile.adjust_info(-1);
                    this.adjust_info(1);
                }
            }
            None => (),
        });
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};

pub fn dirt_moisture(tile: &Tile) -> u8 {
    if tile.element_id() == DIRT.id {
        tile.special_info()
    } else {
        0
    }
}

pub struct DirtSetup;

impl ElementSetup for DirtSetup {
    fn register_reactions(&self, world: &mut crate::world::World) {
        world.register_collision_reaction(&WATER, &DIRT, |water, mut dirt| {
            if dirt.special_info() <= 192 {
                dirt.adjust_info(64);
                (None, Some(dirt))
            } else {
                (Some(water), Some(dirt))
            }
            // Water is absorbed
        });
    }

    fn build_element(&self) -> Element {
        DIRT.clone()
    }

    fn get_id(&self) -> crate::element::ElementId {
        DIRT.id()
    }
}
