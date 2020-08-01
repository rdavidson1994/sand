use crate::{neighbors, Color, Element, FIXED};

static BLACK: Color = [0.1, 0.1, 0.2, 1.0];
static LIGHTEST_BLUE: Color = [0.9, 0.9, 1.0, 1.0];

const GLASS_EDGE: u8 = 1;
const GLASS_INNER: u8 = 2;

#[allow(dead_code)]
pub static GLASS: Element = Element {
    flags: FIXED,
    color: LIGHTEST_BLUE,
    mass: 10,
    id: 10,
    periodic_reaction: Some(|world, i| {
        let mut all_glass_neighbors = true;
        for j in neighbors(i) {
            match &world[j] {
                Some(tile) => {
                    if tile.element_id() != GLASS.id {
                        all_glass_neighbors = false;
                        break;
                    }
                }
                None => {
                    all_glass_neighbors = false;
                    break;
                }
            }
        }
        match &mut world[i] {
            None => {}
            Some(tile) => {
                if all_glass_neighbors {
                    tile.edit_state(GLASS.id(), GLASS_INNER);
                } else {
                    tile.edit_state(GLASS.id(), GLASS_EDGE);
                }
            }
        }
    }),
    state_colors: Some(|state| match state {
        GLASS_INNER => &BLACK,
        GLASS_EDGE | _ => &LIGHTEST_BLUE,
    }),
};
