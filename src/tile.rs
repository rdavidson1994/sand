use crate::{Element, EFlag};
use std::fmt::Display;
use num::Bounded;
use std::convert::TryFrom;
use std::any::type_name;

mod element_state;
pub use element_state::*;

const BASE_RESTITUTION : f64 = 0.5;
const BASE_COLLIDE_RESTITUTION : f64 = 0.8;

#[derive(Clone)]
pub struct Vector {
    pub x : i8,
    pub y : i8,
}

#[derive(Clone)]
pub struct Tile {
    pub paused: bool,
    pub velocity: Vector,
    pub position: Vector,
    element: &'static Element,
    element_data: ElementData,
}

impl Tile {
    pub fn new(
        element: &'static Element,
        element_state: ElementState,
        position: Vector,
        velocity: Vector,
        paused: bool,
    ) -> Tile
    {
        Tile {
            element,
            element_data : ElementData::new(element_state),
            paused,
            position,
            velocity
        }
    }
    pub fn stationary(element: &'static Element, element_state: ElementState) -> Tile {
        Tile{
            element,
            element_data: ElementData::new(element_state),
            paused: false,
            position: Vector {
                x: 0,
                y: 0,
            },
            velocity: Vector {
                x: 0,
                y: 0,
            },
        }
    }

    pub fn set_element(&mut self, element: &'static Element) {
        self.element = element;
        self.element_data = ElementData::none();
    }

    pub fn get_element(&self) -> &'static Element {
        self.element
    }

    pub fn element_id(&self) -> u32 {
        self.element.id
    }

    pub fn edit_state(&mut self, new_state: ElementState) {
        self.element_data.stage(new_state);
    }

    pub fn save_state(&mut self) {
        self.element_data.commit();
    }

    pub fn get_state(&self) -> &ElementState {
        self.element_data.as_ref()
    }

    pub fn color(&self) -> &[f32; 4] {
        &self.element.color
    }

    pub fn has_flag(&self, flag: EFlag) -> bool {
        self.element.has_flag(flag)
    }

    pub fn elastic_collide_y(&mut self, particle2: &mut Tile) {
        let (v1y, v2y) = elastic_collide(
            self.velocity.y,
            particle2.velocity.y,
            self.element.mass,
            particle2.element.mass,
        );
        self.velocity.y = v1y;
        particle2.velocity.y = v2y;
    }

    pub fn elastic_collide_x(&mut self, particle2: &mut Tile) {
        let (v1x, v2x) = elastic_collide(
            self.velocity.x,
            particle2.velocity.x,
            self.element.mass,
            particle2.element.mass,
        );
        self.velocity.x = v1x;
        particle2.velocity.x = v2x;
    }

    pub fn reflect_velocity_x(&mut self) {
        self.velocity.x = (-(self.velocity.x as f64) * BASE_RESTITUTION).trunc() as i8;
    }

    pub fn reflect_velocity_y(&mut self) {
        self.velocity.y = (-(self.velocity.y as f64) * BASE_RESTITUTION).trunc() as i8;
    }

}

fn elastic_collide(v1: i8, v2: i8, m1: i8, m2: i8) -> (i8, i8) {
    let v1 = v1 as f64;
    let v2 = v2 as f64;
    let m1 = m1 as f64;
    let m2 = m2 as f64;
    let new_v1 = (((m1 - m2)/(m1 + m2))*v1 + 2.0*m2/(m1+m2)*v2) * BASE_COLLIDE_RESTITUTION;
    let new_v2 = (((m2 - m1)/(m2 + m1))*v2 + 2.0*m1/(m2+m1)*v1) * BASE_COLLIDE_RESTITUTION;
    (
        clamp_convert::<i32, i8>(new_v1.trunc() as i32),
        clamp_convert::<i32, i8>(new_v2.trunc() as i32),
    )
}

fn clamp_convert<T,V>(t: T) -> V
    where
        T : PartialOrd + Copy + Display,
        V : TryFrom<T> + Bounded + Into<T> + Display,
{
    if let Ok(v) = V::try_from(t) {
        v
    }
    else if t > V::max_value().into() {
        V::max_value()
    }
    else if t < V::min_value().into() {
        V::min_value()
    }
    else {
        panic!(
            "Conversion of {input} from {T} to {V} failed,\
             even though {input} is between {V}::max_value()=={v_max}\
             and {V}::min_value()=={v_min}",
            input = t,
            T = type_name::<T>(),
            V = type_name::<V>(),
            v_max = V::max_value(),
            v_min = V::min_value(),
        )
    }
}

