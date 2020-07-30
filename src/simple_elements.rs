use crate::{Element, FIXED, GRAVITY, NO_FLAGS, PAUSE_EXEMPT};
use rand::{self, Rng};

pub static ELEMENT_DEFAULT: Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.0, 1.0, 1.0],
    mass: 0,
    id: 255,
    periodic_side_effect: None,
    periodic_reaction: None,
    state_colors: None,
};

pub static SAND: Element = Element {
    flags: GRAVITY,
    color: [1.0, 1.0, 0.5, 1.0],
    mass: 10,
    id: 2,
    ..ELEMENT_DEFAULT
};

pub static ROCK: Element = Element {
    flags: GRAVITY,
    color: [0.5, 0.5, 0.5, 1.0],
    mass: 50,
    id: 1,
    ..ELEMENT_DEFAULT
};

pub static GAS: Element = Element {
    flags: PAUSE_EXEMPT,
    color: [1.0, 0.5, 1.0, 1.0],
    mass: 3,
    id: 3,
    ..ELEMENT_DEFAULT
};

pub static WALL: Element = Element {
    flags: FIXED,
    color: [1.0, 1.0, 1.0, 1.0],
    mass: 127,
    id: 0,
    ..ELEMENT_DEFAULT
};
