use crate::element::PERFECT_RESTITUTION;
use crate::world::World;
use crate::{Color, Element, ElementId, ElementSetup, ELEMENT_DEFAULT, FIXED};

const NEUTRAL: u8 = 1;
const NEUTRAL_COLOR: Color = [0.2, 0.2, 0.25, 1.0];

const CHARGED_HEAD: u8 = 2;
const CHARGED_HEAD_COLOR: Color = [0.5, 0.5, 0.8, 1.0];

const CHARGED_TAIL: u8 = 3;
const CHARGED_TAIL_COLOR: Color = [0.3, 0.3, 0.7, 1.0];

pub static METAL: Element = Element {
    mass: 10,
    flags: FIXED,
    id: 7,
    color: NEUTRAL_COLOR,
    state_colors: Some(|special_info| match special_info {
        CHARGED_HEAD => &CHARGED_HEAD_COLOR,
        CHARGED_TAIL => &CHARGED_TAIL_COLOR,
        NEUTRAL | _ => &NEUTRAL_COLOR,
    }),

    periodic_reaction: Some(|world, i| {
        let (this, mut neighbors) = world.mutate_neighbors(i);
        match this.special_info() {
            CHARGED_HEAD => {
                this.edit_state(METAL.id(), CHARGED_TAIL);
            }
            CHARGED_TAIL => {
                this.edit_state(METAL.id(), NEUTRAL);
            }
            NEUTRAL | _ => {
                let mut adjacent_heads = 0;
                neighbors.for_each(|neighbor| {
                    if let Some(tile) = neighbor {
                        if tile.has_state(METAL.id(), CHARGED_HEAD) {
                            adjacent_heads += 1;
                        }
                    }
                });
                if adjacent_heads == 1 || adjacent_heads == 2 {
                    this.edit_state(METAL.id(), CHARGED_HEAD);
                }
            }
        }
    }),
    ..ELEMENT_DEFAULT
};

pub static ELECTRON: Element = Element {
    mass: 2,
    flags: PERFECT_RESTITUTION,
    id: 8,
    color: [0.5, 0.5, 1.0, 1.0],
    ..ELEMENT_DEFAULT
};

pub struct ElectronSetup;
impl ElementSetup for ElectronSetup {
    fn register_reactions(&self, world: &mut World) {
        world.register_collision_side_effect(&ELECTRON, &METAL, |world, i_elec, i_metal| {
            if let Some(metal_tile) = &mut world[i_metal] {
                metal_tile.edit_state(METAL.id(), CHARGED_HEAD);
            }
            world[i_elec] = None;
        });
    }

    fn build_element(&self) -> Element {
        ELECTRON.clone()
    }

    fn get_id(&self) -> ElementId {
        ELECTRON.id()
    }
}
