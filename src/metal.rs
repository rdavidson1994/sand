use crate::element::{PeriodicReaction, FLUID, GRAVITY, PAUSE_EXEMPT, PERFECT_RESTITUTION};
use crate::world::World;
use crate::{tile::Tile, Color, Element, ElementId, ElementSetup, ELEMENT_DEFAULT, FIXED};
use std::cmp;

pub(crate) const NEUTRAL: u8 = 1;
const NEUTRAL_COLOR: Color = [0.2, 0.2, 0.25, 1.0];

pub const CHARGED_HEAD: u8 = 255;
const CHARGED_HEAD_COLOR: Color = [0.5, 0.5, 0.8, 1.0];

const CHARGED_TAIL: u8 = 2;
const CHARGED_TAIL_COLOR: Color = [0.3, 0.3, 0.7, 1.0];

const LIQUID_COLOR: Color = [0.7, 0.3, 0.5, 1.0];
const METAL_MELT_TEMPERATURE : i16 = 1500;

impl Tile {
    pub fn is_charged_metal(&self) -> bool {
        self.element_id() == METAL.id && self.special_info() > 2
    }
}

pub static METAL: Element = Element {
    mass: 10,
    flags: FIXED,
    id: 7,
    color: NEUTRAL_COLOR,
    state_colors: Some(|special_info| match special_info {
        CHARGED_TAIL => &CHARGED_TAIL_COLOR,
        NEUTRAL => &NEUTRAL_COLOR,
        _ => &CHARGED_HEAD_COLOR,
    }),

    periodic_reaction: PeriodicReaction::Some(|mut this, world| {
        if this.temperature > METAL_MELT_TEMPERATURE {
            this.set_element(LIQUID_METAL.id())
        }
        match this.special_info() {
            CHARGED_TAIL => {
                this.edit_state(METAL.id(), NEUTRAL);
            }
            NEUTRAL => {
                let mut adjacent_heads = 0;
                let mut min_charge = 255;
                for i in world.neighbors() {
                    if let Some(tile) = &world[i] {
                        if tile.is_charged_metal() {
                            adjacent_heads += 1;
                            min_charge = cmp::min(min_charge, tile.special_info() - 1)
                        }
                    }
                }
                if adjacent_heads == 1 || adjacent_heads == 2 && min_charge > 2 {
                    this.edit_state(METAL.id(), min_charge);
                }
            }
            _ => {
                this.edit_state(METAL.id(), CHARGED_TAIL);
            }
        }
        Some(this)
    }),
    ..ELEMENT_DEFAULT
};

pub static LIQUID_METAL: Element = Element {
    flags: FLUID | GRAVITY | PAUSE_EXEMPT,
    color: LIQUID_COLOR,
    mass: 10,
    id: 17,
    periodic_reaction: PeriodicReaction::Some(|mut this, _world| {
        if this.temperature < METAL_MELT_TEMPERATURE - 1 {
            this.set_element(METAL.id())
        }
        Some(this)
    }),
    default_temperature: METAL_MELT_TEMPERATURE + 20,
    ..ELEMENT_DEFAULT
};

pub static ELECTRON: Element = Element {
    mass: 2,
    flags: PERFECT_RESTITUTION,
    id: 8,
    color: [0.5, 0.5, 1.0, 1.0],
    periodic_reaction: PeriodicReaction::DecayToNothing {
        lifetime: 8,
        rarity: 8,
    },
    default_temperature: 300,
    ..ELEMENT_DEFAULT
};

pub struct ElectronSetup;
impl ElementSetup for ElectronSetup {
    fn register_reactions(&self, world: &mut World) {
        world.register_collision_side_effect(&METAL, &ELECTRON, |mut metal, _electron, _world| {
            metal.edit_state(METAL.id(), CHARGED_HEAD);
            (Some(metal), None)
        });
    }

    fn build_element(&self) -> Element {
        ELECTRON.clone()
    }

    fn get_id(&self) -> ElementId {
        ELECTRON.id()
    }
}
