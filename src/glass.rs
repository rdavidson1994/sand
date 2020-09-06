use crate::{element::PeriodicReaction, Color, Element, FIXED};

static BLACK: Color = [0.1, 0.1, 0.2, 1.0];
static LIGHTEST_BLUE: Color = [0.9, 0.9, 1.0, 1.0];

const GLASS_EDGE: u8 = 1;
const GLASS_INNER: u8 = 2;

pub static GLASS: Element = Element {
    flags: FIXED,
    color: LIGHTEST_BLUE,
    mass: 10,
    id: 10,
    periodic_reaction: PeriodicReaction::Some(|mut this, world| {
        for j in world.neighbors() {
            if world[j]
                .clone()
                .filter(|tile| tile.element_id() == GLASS.id)
                .is_none()
            // If any neighboring tile is empty or non-glass
            {
                this.edit_state(GLASS.id(), GLASS_EDGE);
                // This tile's state becomes "GLASS_EDGE"
                return Some(this);
            };
        }
        // Otherwise its state becomes "GLASS_INNER"
        this.edit_state(GLASS.id(), GLASS_INNER);
        Some(this)
    }),
    state_colors: Some(|state| match state {
        GLASS_INNER => &BLACK,
        _ => &LIGHTEST_BLUE,
    }),
};
