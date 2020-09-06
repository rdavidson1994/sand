use crate::tile::Tile;
use crate::world::World;
use crate::world_view::NeighborhoodView;
use crate::ELEMENTS;
use std::num::NonZeroU8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ElementId(pub u8);

impl ElementId {
    pub fn get_element(self) -> &'static Element {
        &ELEMENTS[self.0 as usize]
    }
}

// Can't use bitflags crate at the moment, since we need FLAG1 | FLAG2 to be const
pub type EFlag = u8;

pub const NO_FLAGS: EFlag = 0;
pub const GRAVITY: EFlag = 1 << 0;
pub const FIXED: EFlag = 1 << 1;
pub const PAUSE_EXEMPT: EFlag = 1 << 2;
pub const PERFECT_RESTITUTION: EFlag = 1 << 3;
pub const FLUID: EFlag = 1 << 4;

impl SpecialElementInfo {
    pub fn none() -> Self {
        Self::new(1)
    }

    pub fn new(byte: u8) -> Self {
        SpecialElementInfo(NonZeroU8::new(byte).unwrap())
    }

    pub fn as_u8(self) -> u8 {
        self.0.get() as u8
    }
}

pub trait ElementSetup: Sync {
    fn register_reactions(&self, world: &mut World);
    fn build_element(&self) -> Element;
    fn get_id(&self) -> ElementId;
}

pub type Color = [f32; 4];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpecialElementInfo(NonZeroU8);

pub struct DefaultSetup {
    element: &'static Element,
}

impl DefaultSetup {
    pub fn new(element: &'static Element) -> Self {
        DefaultSetup { element }
    }
}

impl ElementSetup for DefaultSetup {
    fn register_reactions(&self, _world: &mut World) {
        // Do nothing
    }

    fn build_element(&self) -> Element {
        self.element.clone()
    }

    fn get_id(&self) -> ElementId {
        self.element.id()
    }
}

#[derive(Clone, Copy)]
pub enum PeriodicReaction {
    Some(fn(Tile, NeighborhoodView<Option<Tile>>) -> Option<Tile>),
    None,
    DecayInto {
        element_id: ElementId,
        lifetime: u8,
        rarity: i32,
    },
    DecayToNothing {
        lifetime: u8,
        rarity: i32,
    },
}

#[derive(Clone)]
pub struct Element {
    pub flags: EFlag,
    pub color: Color,
    pub mass: i8,
    pub id: u8,
    pub periodic_reaction: PeriodicReaction,
    pub state_colors: Option<fn(u8) -> &'static Color>,
}

impl Element {
    pub fn has_flag(&self, flag: EFlag) -> bool {
        flag & self.flags != 0
    }

    pub const fn id(&self) -> ElementId {
        ElementId(self.id)
    }

    pub fn get_color(&self, special_info: u8) -> &[f32; 4] {
        match self.state_colors {
            Some(function) => function(special_info),
            None => &self.color,
        }
    }
}
