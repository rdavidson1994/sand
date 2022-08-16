use num::Bounded;
use std::any::type_name;
use std::convert::TryFrom;
use std::fmt::Display;

mod element_state;
use crate::element::{EFlag, Element, ElementId, SpecialElementInfo, PERFECT_RESTITUTION};
use crate::ELEMENTS;
pub use element_state::*;

const BASE_RESTITUTION: f64 = 0.5;
const BASE_COLLIDE_RESTITUTION: f64 = 0.8;

#[derive(Clone, Copy)]
pub struct Vector {
    pub x: i8,
    pub y: i8,
}

impl Vector {
    pub fn is_zero(&self) -> bool {
        self.x == 0 && self.y == 0
    }
}

#[derive(Clone)]
pub struct Tile {
    //pub paused: bool,
    pub velocity: Vector,
    pub position: Vector,
    // Celcius (for now)
    pub temperature: i16,
    element_data: ElementData,
}

impl Tile {
    pub fn new(
        element_state: ElementState,
        position: Vector,
        velocity: Vector,
        temperature: i16,
        //paused: bool,
    ) -> Tile {
        Tile {
            element_data: ElementData::new(element_state),
            //paused,
            temperature,
            position,
            velocity,
        }
    }
    pub fn stationary(element_state: ElementState, temperature: i16) -> Tile {
        Tile {
            element_data: ElementData::new(element_state),
            temperature,
            //paused: false,
            position: Vector { x: 0, y: 0 },
            velocity: Vector { x: 0, y: 0 },
        }
    }

    pub fn set_element(&mut self, element: ElementId) {
        self.element_data.stage(ElementState::default(element)); // = ElementData::new(ElementState::new(element));
    }

    pub fn get_element(&self) -> &'static Element {
        &ELEMENTS[self.element_id() as usize]
        //self.element
    }

    pub fn element_id(&self) -> u8 {
        self.element_data.element_id().0
    }

    pub fn special_info(&self) -> u8 {
        self.get_state().special_info.as_u8()
    }

    pub fn edit_state(&mut self, element_id: ElementId, special_info: u8) {
        self.element_data.stage(ElementState {
            element_id,
            special_info: SpecialElementInfo::new(special_info),
        });
    }

    pub fn adjust_info(&mut self, delta: i16) {
        self.element_data.adjust(delta);
    }

    pub fn increment_info(&mut self) {
        self.adjust_info(1);
    }

    pub fn decrement_info(&mut self) {
        self.adjust_info(-1);
    }

    pub fn has_state(&self, element_id: ElementId, special_info: u8) -> bool {
        *self.get_state() == ElementState::new(element_id, special_info)
    }

    pub fn save_state(&mut self) {
        self.element_data.commit();
    }

    pub fn get_state(&self) -> &ElementState {
        self.element_data.as_ref()
    }

    pub fn color(&self) -> &[f32; 4] {
        let state = self.get_state();
        state
            .element_id
            .get_element()
            .get_color(state.special_info.as_u8())
    }

    pub fn has_flag(&self, flag: EFlag) -> bool {
        self.get_element().has_flag(flag)
    }

    pub fn elastic_collide_y(&mut self, particle2: &mut Tile) {
        let (v1y, v2y) = elastic_collide(
            self.velocity.y,
            particle2.velocity.y,
            self.get_element().mass,
            particle2.get_element().mass,
        );
        if self.has_flag(PERFECT_RESTITUTION) {
            self.velocity.y = v1y;
        } else {
            self.velocity.y = (v1y as f64 * BASE_COLLIDE_RESTITUTION).trunc() as i8;
        }
        if particle2.has_flag(PERFECT_RESTITUTION) {
            particle2.velocity.y = v2y;
        } else {
            particle2.velocity.y = (v2y as f64 * BASE_COLLIDE_RESTITUTION).trunc() as i8;
        }
    }

    pub fn elastic_collide_x(&mut self, particle2: &mut Tile) {
        let (v1x, v2x) = elastic_collide(
            self.velocity.x,
            particle2.velocity.x,
            self.get_element().mass,
            particle2.get_element().mass,
        );
        if self.has_flag(PERFECT_RESTITUTION) {
            self.velocity.x = v1x;
        } else {
            self.velocity.x = (v1x as f64 * BASE_COLLIDE_RESTITUTION).trunc() as i8;
        }
        if particle2.has_flag(PERFECT_RESTITUTION) {
            particle2.velocity.x = v2x;
        } else {
            particle2.velocity.x = (v2x as f64 * BASE_COLLIDE_RESTITUTION).trunc() as i8;
        }
    }

    pub fn reflect_velocity_x(&mut self) {
        if self.has_flag(PERFECT_RESTITUTION) {
            self.velocity.x = -self.velocity.x;
        } else {
            self.velocity.x = (-(self.velocity.x as f64) * BASE_RESTITUTION).trunc() as i8;
        }
    }

    pub fn reflect_velocity_y(&mut self) {
        if self.has_flag(PERFECT_RESTITUTION) {
            self.velocity.y = -self.velocity.y;
        } else {
            self.velocity.y = (-(self.velocity.y as f64) * BASE_RESTITUTION).trunc() as i8;
        }
    }
}

fn elastic_collide(v1: i8, v2: i8, m1: i8, m2: i8) -> (i8, i8) {
    let v1 = v1 as f64;
    let v2 = v2 as f64;
    let m1 = m1 as f64;
    let m2 = m2 as f64;
    let new_v1 =
        (((m1 - m2) / (m1 + m2)) * v1 + 2.0 * m2 / (m1 + m2) * v2) * BASE_COLLIDE_RESTITUTION;
    let new_v2 =
        (((m2 - m1) / (m2 + m1)) * v2 + 2.0 * m1 / (m2 + m1) * v1) * BASE_COLLIDE_RESTITUTION;
    (
        clamp_convert::<i32, i8>(new_v1.trunc() as i32),
        clamp_convert::<i32, i8>(new_v2.trunc() as i32),
    )
}

fn clamp_convert<Source, Target>(t: Source) -> Target
where
    Source: PartialOrd + Copy + Display,
    Target: TryFrom<Source> + Bounded + Into<Source> + Display,
{
    if let Ok(v) = Target::try_from(t) {
        v
    } else if t > Target::max_value().into() {
        Target::max_value()
    } else if t < Target::min_value().into() {
        Target::min_value()
    } else {
        panic!(
            "Conversion of {input} from {Source} to {Target} failed,\
             even though {input} is between {Target}::max_value()=={v_max}\
             and {Target}::min_value()=={v_min}",
            input = t,
            Source = type_name::<Source>(),
            Target = type_name::<Target>(),
            v_max = Target::max_value(),
            v_min = Target::min_value(),
        )
    }
}
