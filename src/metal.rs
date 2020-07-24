use crate::world::World;
use crate::{
    for_neighbors, neighbors, Element, ElementId, ElementSetup, ELEMENT_DEFAULT, FIXED, GAS,
    NO_FLAGS,
};

pub static METAL: Element = Element {
    mass: 10,
    flags: FIXED,
    id: 7,
    color: [0.2, 0.2, 0.25, 1.0],
    periodic_side_effect: Some(|world, i| {
        let mut head_neighbor_count = 0;
        for j in neighbors(i) {
            if let Some(tile) = &world[j] {
                if tile.get_element().id() == METAL_CHARGED_HEAD.id() {
                    head_neighbor_count += 1;
                }
            }
        }
        if head_neighbor_count > 0 {
            println!("{}", head_neighbor_count);
        }
        if head_neighbor_count == 1 || head_neighbor_count == 2 {
            world[i]
                .as_mut()
                .unwrap()
                .set_element(METAL_CHARGED_HEAD.id());
        }
    }),
    ..ELEMENT_DEFAULT
};

pub static METAL_CHARGED_HEAD: Element = Element {
    color: [0.5, 0.5, 0.8, 1.0],
    mass: 10,
    flags: FIXED,
    id: 9,
    periodic_reaction: Some(|tile| tile.set_element(METAL_CHARGED_TAIL.id())),
    ..ELEMENT_DEFAULT
};

pub static METAL_CHARGED_TAIL: Element = Element {
    color: [0.3, 0.3, 0.7, 1.0],
    mass: 10,
    flags: FIXED,
    id: 10,
    periodic_reaction: Some(|tile| tile.set_element(METAL.id())),
    ..ELEMENT_DEFAULT
};

pub static ELECTRON: Element = Element {
    mass: 2,
    flags: NO_FLAGS,
    id: 8,
    color: [0.5, 0.5, 1.0, 1.0],
    ..ELEMENT_DEFAULT
};

pub struct ElectronSetup;
impl ElementSetup for ElectronSetup {
    fn register_reactions(&self, world: &mut World) {
        world.register_collision_reaction(&ELECTRON, &METAL, |elec_tile, metal_tile| {
            metal_tile.set_element(METAL_CHARGED_HEAD.id());
            elec_tile.set_element(GAS.id());
        });
    }

    fn build_element(&self) -> Element {
        ELECTRON.clone()
    }

    fn get_id(&self) -> ElementId {
        ELECTRON.id()
    }
}
