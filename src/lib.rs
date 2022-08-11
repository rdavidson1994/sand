#![allow(clippy::new_without_default)]
mod app;
mod conway;
mod element;
mod element_menu;
mod fire;
mod gas;
mod glass;
mod glue;
mod lava;
mod metal;
mod oil;
mod simple_elements;
mod snow;
mod tile;
mod util;
mod water;
mod world;
mod world_view;

use crate::app::App;
use crate::conway::CONWAY;
use crate::element::{Color, DefaultSetup, Element, ElementId, ElementSetup, FIXED};
use crate::element_menu::ElementMenu;
use crate::fire::{FireElementSetup, ASH, FIRE};
use crate::gas::{GasSetup, GAS};
use crate::glass::GLASS;
use crate::glue::{GlueSetup, SOLID_GLUE};
use crate::lava::LavaSetup;
use crate::metal::{ElectronSetup, LIQUID_METAL, METAL};
use crate::oil::OIL;
use crate::simple_elements::{ELEMENT_DEFAULT, ROCK, SAND, WALL};
use crate::tile::{ElementState, Tile, Vector};
use crate::water::{STEAM, WATER};
use crate::world::World;
use itertools::{iproduct, Itertools};
use lazy_static::{self as lazy_static_crate, lazy_static};
use rand::{thread_rng, Rng};
use std::collections::VecDeque;

type SetupList = Vec<Box<dyn ElementSetup>>;
type SetupSlice<'a> = &'a [Box<dyn ElementSetup>];

lazy_static! {
    pub static ref SETUPS: SetupList = {
        let default_setup = |x| Box::new(DefaultSetup::new(x));
        vec![
            default_setup(&SAND),
            default_setup(&ROCK),
            default_setup(&WALL),
            default_setup(&WATER),
            default_setup(&ASH),
            default_setup(&METAL),
            default_setup(&GLASS),
            Box::new(GasSetup),
            Box::new(LavaSetup),
            Box::new(ElectronSetup),
            Box::new(FireElementSetup),
            Box::new(SnowSetup),
            default_setup(&OIL),
            default_setup(&CONWAY),
            Box::new(GlueSetup),
            default_setup(&SOLID_GLUE),
            default_setup(&STEAM),
            default_setup(&LIQUID_METAL),
        ]
    };
}

lazy_static! {
    pub static ref ELEMENTS: Vec<Element> = {
        let mut out = vec![];
        for s in SETUPS.iter().sorted_by_key(|x| x.get_id().0) {
            out.push(s.build_element());
        }
        for (i, elem) in out.iter().enumerate() {
            assert_eq!(i, elem.id as usize);
        }
        out
    };
}

const MENU_PIXEL_HEIGHT: i32 = 70;
const WORLD_WIDTH: i32 = 200;
const WORLD_HEIGHT: i32 = 200;
const WORLD_SIZE: i32 = WORLD_HEIGHT * WORLD_WIDTH;
const TILE_PIXELS: i32 = 3;
const WINDOW_PIXEL_WIDTH: i32 = WORLD_WIDTH * TILE_PIXELS;
const PLAY_AREA_PIXEL_HEIGHT: i32 = WORLD_HEIGHT * TILE_PIXELS;
const WINDOW_PIXEL_HEIGHT: i32 = PLAY_AREA_PIXEL_HEIGHT + MENU_PIXEL_HEIGHT;
const UPDATES_PER_FRAME: i32 = 20;
// 1 frame = 20 updates
// 1 second = 60 frames = 1200 updates
const GRAVITY_PERIOD: i32 = 5;
const REACTION_PERIOD: i32 = 3; // This is still fast! :D It used to be 100!
const PAUSE_VELOCITY: i8 = 3;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use crate::snow::SnowSetup;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{ButtonEvent, MouseCursorEvent, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < WORLD_WIDTH && y >= 0 && y < WORLD_HEIGHT
}

#[inline]
fn below(position: usize) -> Option<usize> {
    position
        .checked_add(WORLD_WIDTH as usize)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn above(position: usize) -> Option<usize> {
    position
        .checked_sub(WORLD_WIDTH as usize)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn left(position: usize) -> Option<usize> {
    position
        .checked_sub(1)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn right(position: usize) -> Option<usize> {
    position
        .checked_add(1)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn adjacent_x(position1: usize, position2: usize) -> bool {
    let is_left = match left(position1) {
        None => false,
        Some(x) => x == position2,
    };
    let is_right = match right(position1) {
        None => false,
        Some(x) => x == position2,
    };
    is_left || is_right
}

#[inline]
#[allow(dead_code)]
fn adjacent_y(position1: usize, position2: usize) -> bool {
    let is_above = match above(position1) {
        None => false,
        Some(x) => x == position2,
    };
    let is_below = match below(position1) {
        None => false,
        Some(x) => x == position2,
    };
    is_above || is_below
}

pub fn neighbor_count(index: usize, predicate: impl Fn(usize) -> bool) -> usize {
    neighbors(index).filter(|&x| predicate(x)).count()
}

pub fn neighbors(index: usize) -> impl Iterator<Item = usize> + 'static {
    let x = index as i32 % WORLD_WIDTH;
    let y = index as i32 / WORLD_WIDTH;
    iproduct!(-1i32..=1i32, -1i32..=1i32) // consider all adjacent tuples
        .filter(|&tuple| tuple != (0, 0)) // exclude same tile
        .map(move |(dx, dy)| (x + dx, y + dy))
        .filter(|&(x, y)| in_bounds(x, y)) // exclude tiles outside world bounds
        .map(|(x, y)| (x + y * WORLD_WIDTH) as usize) // calculate index
}

pub fn raw_neighbors(index: usize) -> impl Iterator<Item = usize> + 'static {
    let x = index as i32 % WORLD_WIDTH;
    let y = index as i32 / WORLD_WIDTH;
    iproduct!(-1i32..=1i32, -1i32..=1i32) // consider all adjacent tuples
        .filter(|&tuple| tuple != (0, 0)) // exclude same tile
        .map(move |(dx, dy)| (x + dx, y + dy)) // exclude tiles outside world bounds
        .map(|(x, y)| (x + y * WORLD_WIDTH) as usize) // calculate index
}

fn apply_velocity(world: &mut World, motion_queue: &mut VecDeque<(usize, usize)>) -> bool {
    let mut needs_update = false;
    // This makes more sense at the end, but borrowck didn't like it
    // maybe check it later?
    motion_queue.clear();
    for i in 0..WORLD_SIZE as usize {
        if let Some(ref mut tile) = &mut world[i] {
            if
            /* !tile.paused && */
            !tile.has_flag(FIXED) {
                let (new_x, overflowed_x) = tile.position.x.overflowing_add(tile.velocity.x);
                let (new_y, overflowed_y) = tile.position.y.overflowing_add(tile.velocity.y);
                tile.position.x = new_x;
                tile.position.y = new_y;
                if overflowed_x || overflowed_y {
                    needs_update = true;
                    let delta_x = if overflowed_x {
                        tile.velocity.x.signum()
                    } else {
                        0
                    };
                    let delta_y = if overflowed_y {
                        tile.velocity.y.signum()
                    } else {
                        0
                    };
                    let (old_grid_x, old_grid_y) = coords(i);
                    let (new_grid_x, new_grid_y) =
                        (old_grid_x + delta_x as i32, old_grid_y + delta_y as i32);
                    if in_bounds(new_grid_x, new_grid_y) {
                        let swap_pair = (i, point(new_grid_x, new_grid_y));
                        // this logic is to allow "trains" of adjacent particles
                        // to travel smoothly and not knock each other
                        if delta_y < 0 || (delta_y == 0 && delta_x < 0) {
                            motion_queue.push_back(swap_pair);
                        } else {
                            motion_queue.push_front(swap_pair);
                        }
                    }
                }
            }
        }
    }

    for (i, j) in motion_queue {
        world.move_particle(*i, *j);
    }

    needs_update
}

fn coords(i: usize) -> (i32, i32) {
    (
        (i % (WORLD_WIDTH as usize)) as i32,
        (i / (WORLD_WIDTH as usize)) as i32,
    )
}

fn point(x: i32, y: i32) -> usize {
    (x + y * WORLD_WIDTH) as usize
}

pub trait Pen {
    fn draw(&mut self, world: &mut World, x: f64, y: f64);
    fn get_radius(&self) -> i32;
    fn set_radius(&mut self, radius: i32);
}

pub struct DeletePen {
    radius: i32,
}

impl Pen for DeletePen {
    fn draw(&mut self, world: &mut World, x: f64, y: f64) {
        let x = x.trunc() as i32 / TILE_PIXELS;
        let y = y.trunc() as i32 / TILE_PIXELS;
        for x in x - self.radius..=x + self.radius {
            for y in y - self.radius..=y + self.radius {
                if in_bounds(x, y) {
                    match &world[point(x, y)] {
                        Some(tile) if tile.element_id() != WALL.id => {
                            // Destroy non-wall tiles
                            world[point(x, y)] = None
                        }
                        _ => {
                            // Leave walls and empty tiles alone
                        }
                    }
                }
            }
        }
    }

    fn get_radius(&self) -> i32 {
        self.radius
    }

    fn set_radius(&mut self, radius: i32) {
        self.radius = radius
    }
}

pub struct ElementPen {
    element: &'static Element,
    radius: i32,
}

impl Pen for ElementPen {
    fn draw(&mut self, world: &mut World, x: f64, y: f64) {
        let x = x.trunc() as i32 / TILE_PIXELS;
        let y = y.trunc() as i32 / TILE_PIXELS;
        for x in x - self.radius..=x + self.radius {
            for y in y - self.radius..=y + self.radius {
                let velocity = if self.element.has_flag(FIXED) {
                    Vector { x: 0, y: 0 }
                } else {
                    Vector {
                        x: thread_rng().gen_range(-20, 21),
                        y: thread_rng().gen_range(-20, 21),
                    }
                };
                if in_bounds(x, y) && world[point(x, y)].is_none() {
                    world[point(x, y)] = Some(Tile::new(
                        ElementState::default(self.element.id()),
                        Vector { x: 0, y: 0 },
                        velocity,
                        self.element.default_temperature, //false,
                    ))
                }
            }
        }
    }

    fn get_radius(&self) -> i32 {
        self.radius
    }

    fn set_radius(&mut self, radius: i32) {
        self.radius = radius
    }
}

pub fn game_loop() {
    // Prepare the list of elements and their setup strcuts
    lazy_static_crate::initialize(&SETUPS);
    lazy_static_crate::initialize(&ELEMENTS);
    let elem_count = SETUPS.len();

    // Create the world
    let mut world = World::new(elem_count);

    // Register each element's collision reactions based on setup structs
    for s in SETUPS.iter() {
        s.register_reactions(&mut world);
    }

    // Draw walls around the edge of the playing area
    util::create_walls(&mut world);

    // Create a new Glutin window.
    let open_gl = OpenGL::V3_2;
    let size = [WINDOW_PIXEL_WIDTH as u32, WINDOW_PIXEL_HEIGHT as u32];
    let mut window: Window = WindowSettings::new("Falling sand", size)
        .graphics_api(open_gl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create the app object to store our game state
    let mut app = App::new(
        GlGraphics::new(open_gl),
        world,
        ElementMenu::new(SETUPS.as_ref(), 12),
        Box::new(ElementPen {
            element: &SAND,
            radius: 0,
        }),
    );

    // Set up the piston event loop
    let mut events = Events::new(EventSettings::new())
        .max_fps(60)
        .ups(60)
        .ups_reset(10);

    // Map events we care about to the relevant handlers in our app
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(args) = e.button_args() {
            app.button(&args);
        }

        if let Some(args) = e.mouse_cursor_args() {
            app.mouse_cursor(&args);
        }
    }
}
