mod fire;
mod sand;
mod tile;
mod util;
mod world;

use crate::fire::{FireElementSetup, FIRE};
use crate::sand::{SandSetup, SAND};
use crate::tile::{ElementState, Tile, Vector};
use itertools::{iproduct, Itertools};
use lazy_static::{self as lazy_static_crate, lazy_static};
use rand::{thread_rng, Rng};
use std::collections::VecDeque;
const WORLD_WIDTH: i32 = 120;
const WORLD_HEIGHT: i32 = 120;
const WORLD_SIZE: i32 = WORLD_HEIGHT * WORLD_WIDTH;
const TILE_PIXELS: i32 = 3;
const WINDOW_PIXEL_WIDTH: i32 = WORLD_WIDTH * TILE_PIXELS;
const WINDOW_PIXEL_HEIGHT: i32 = WORLD_HEIGHT * TILE_PIXELS;
const LOGICAL_FRAMES_PER_DISPLAY_FRAME: i32 = 10;
const GRAVITY_PERIOD: i32 = 20;
const DECAY_REACTION_PERIOD: i32 = 100;
const PAUSE_VELOCITY: i8 = 3;
const SECONDS_PER_LOGICAL_FRAME: f64 = 1.0 / 1400.0; // Based on square = 1inch
                                                     //graphics imports
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use crate::world::World;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, ButtonEvent, ButtonState, Key, MouseButton, MouseCursorEvent, RenderArgs, RenderEvent,
    UpdateArgs, UpdateEvent,
};
use piston::window::WindowSettings;
use std::num::NonZeroU8;

#[derive(Clone, Copy)]
pub struct ElementId(u8);

#[derive(Clone)]
pub struct SpecialElementInfo(NonZeroU8);

pub trait ElementSetup: Sync {
    fn register_reactions(&self, world: &mut World);
    fn build_element(&self) -> Element;
    fn get_id(&self) -> ElementId;
}

// Can't use bitflags crate at the moment, since we need FLAG1 | FLAG2 to be const
type EFlag = u8;
const NO_FLAGS: EFlag = 0;
const GRAVITY: EFlag = 1 << 0;
const FIXED: EFlag = 1 << 1;
const PAUSE_EXEMPT: EFlag = 1 << 2;

#[derive(Default, Clone)]
pub struct Element {
    flags: EFlag,
    color: [f32; 4],
    mass: i8,
    id: u8,
    decay_reaction: Option<fn(&mut World, usize)>,
}

impl Element {
    fn has_flag(&self, flag: EFlag) -> bool {
        flag & self.flags != 0
    }

    fn id(&self) -> ElementId {
        ElementId(self.id)
    }
}

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

pub fn for_neighbors(index: usize, mut f: impl FnMut(usize)) {
    let x = index as i32 % WORLD_WIDTH;
    let y = index as i32 / WORLD_WIDTH;
    iproduct!(-1i32..=1, -1i32..=1) // consider all adjacent tuples
        .filter(|&tuple| tuple != (0, 0)) // exclude same tile
        .map(|(dx, dy)| (x + dx, y + dy))
        .filter(|&(x,y)|  // exclude tiles outside world bounds
            in_bounds(x,y))
        .map(|(x, y)| (x + y * WORLD_WIDTH) as usize) // calculate index
        .for_each(|minimum| f(minimum)); // apply input function
}

#[test]
fn for_neighbors_test() {
    let mut results = std::collections::HashSet::<usize>::new();
    for_neighbors(point(5, 5), |i| {
        assert!(results.insert(i));
    });
    assert!(results.contains(&point(4, 4)));
    assert!(results.contains(&point(4, 5)));
    assert!(results.contains(&point(4, 6)));
    assert!(results.contains(&point(5, 4)));
    // The center (5,5), is not included
    assert!(results.contains(&point(5, 6)));
    assert!(results.contains(&point(6, 4)));
    assert!(results.contains(&point(6, 5)));
    assert!(results.contains(&point(6, 6)));
    let mut count = 0;
    for _ in results.drain() {
        count += 1
    }
    assert_eq!(count, 8);
}

fn apply_velocity(world: &mut World, motion_queue: &mut VecDeque<(usize, usize)>) -> bool {
    let mut needs_update = false;
    // This makes more sense at the end, but borrowck didn't like it
    // maybe check it later?
    motion_queue.clear();
    for i in 0..WORLD_SIZE as usize {
        if let Some(ref mut tile) = &mut world[i] {
            if !tile.paused {
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
        assert!(in_bounds(coords(*i).0, coords(*i).1));
        assert!(in_bounds(coords(*j).0, coords(*j).1));
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

static WALL: Element = Element {
    flags: FIXED,
    color: [1.0, 1.0, 1.0, 1.0],
    mass: 127,
    id: 0,
    decay_reaction: None,
};

struct WallSetup;
impl ElementSetup for WallSetup {
    fn register_reactions(&self, _world: &mut World) {}
    fn get_id(&self) -> ElementId {
        WALL.id()
    }
    fn build_element(&self) -> Element {
        WALL.clone()
    }
}

static ROCK: Element = Element {
    flags: GRAVITY,
    color: [0.5, 0.5, 0.5, 1.0],
    mass: 50,
    id: 1,
    decay_reaction: None,
};

// TODO move to file
struct RockSetup;
impl ElementSetup for RockSetup {
    fn register_reactions(&self, _world: &mut World) {}
    fn get_id(&self) -> ElementId {
        ROCK.id()
    }
    fn build_element(&self) -> Element {
        ROCK.clone()
    }
}

static GAS: Element = Element {
    flags: PAUSE_EXEMPT,
    color: [1.0, 0.5, 1.0, 1.0],
    mass: 3,
    id: 3,
    decay_reaction: None,
};

static WATER: Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT,
    color: [0.0, 0.0, 1.0, 1.0],
    mass: 8,
    id: 6,
    decay_reaction: Some(|w, p| {
        // Water "jiggles" slightly
        let water_tile = w[p].as_mut().unwrap();
        water_tile.velocity.x += thread_rng().gen_range(-3, 4);
    }),
};

type CollisionSideEffect = fn(&mut World, usize, usize);
type CollisionReaction = fn(&mut Tile, &mut Tile);

struct App {
    gl: GlGraphics,
    time_balance: f64,
    frame_balance: i32,
    turn: i32,
    world: World,
    motion_queue: VecDeque<(usize, usize)>,
    needs_render: bool,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        //println!("FPS: {}", 1.0/args.ext_dt);
        if !self.needs_render {
            return;
        }
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let world_ref = &self.world;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            let transform = c.transform;
            for i in 0..WORLD_SIZE as usize {
                if let Some(tile) = &world_ref[i] {
                    let (x, y) = coords(i);
                    let square = rectangle::square(
                        (x * TILE_PIXELS) as f64,
                        (y * TILE_PIXELS) as f64,
                        TILE_PIXELS as f64,
                    );
                    rectangle(*tile.color(), square, transform, gl);
                }
            }
        });
        self.needs_render = false;
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.time_balance += args.dt;
        let frames_to_render = self.time_balance / SECONDS_PER_LOGICAL_FRAME;
        let mut i = 0;
        while i < frames_to_render.trunc() as i32 {
            self.world.pause_particles();
            if self.turn % GRAVITY_PERIOD == 0 {
                self.world.apply_gravity();
            }
            if self.turn % DECAY_REACTION_PERIOD == 0 {
                self.world.apply_decay_reactions();
            }
            apply_velocity(&mut self.world, &mut self.motion_queue);
            self.turn += 1;
            i += 1;
        }
        self.time_balance -= (i as f64) * SECONDS_PER_LOGICAL_FRAME;
        self.frame_balance += i;
        if self.frame_balance > LOGICAL_FRAMES_PER_DISPLAY_FRAME {
            self.needs_render = true;
            self.frame_balance = 0;
        }
    }
}

trait Pen {
    fn draw(&mut self, world: &mut World, x: f64, y: f64);
}

struct ElementPen {
    element: &'static Element,
}

impl Pen for ElementPen {
    fn draw(&mut self, world: &mut World, x: f64, y: f64) {
        let x = x.trunc() as i32 / TILE_PIXELS;
        let y = y.trunc() as i32 / TILE_PIXELS;
        if in_bounds(x, y) && world[point(x, y)].is_none() {
            world[point(x, y)] = Some(Tile::new(
                ElementState::new(self.element.id()),
                Vector { x: 0, y: 0 },
                Vector {
                    x: thread_rng().gen_range(-20, 21),
                    y: thread_rng().gen_range(-20, 21),
                },
                false,
            ))
        }
    }
}

lazy_static! {
    static ref SETUPS: Vec<Box<dyn ElementSetup>> = {
        vec![
            Box::new(SandSetup),
            Box::new(RockSetup),
            Box::new(WallSetup),
            // Todo: add the rest of the elements
        ]
    };
}

lazy_static! {
    static ref ELEMENTS: Vec<Element> = {
        let mut out = vec![];
        for s in SETUPS.iter().sorted_by_key(|x| x.get_id().0) {
            out.push(s.build_element());
        }
        for (i, elem) in out.iter().enumerate() {
            //assert_eq!(i, elem.id as usize);
        }
        out
   };
}

pub fn game_loop() {
    lazy_static_crate::initialize(&SETUPS);
    lazy_static_crate::initialize(&ELEMENTS);
    let mut world = World::new();
    for s in SETUPS.iter() {
        s.register_reactions(&mut world);
    }

    let open_gl = OpenGL::V3_2;
    let size = [WINDOW_PIXEL_WIDTH as u32, WINDOW_PIXEL_HEIGHT as u32];

    //let mut i = 0;
    util::create_walls(&mut world);
    util::populate_world_water_bubble(&mut world);
    //FireElementSetup.register_reactions(&mut world);

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Falling sand", size)
        .graphics_api(open_gl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        world,
        gl: GlGraphics::new(open_gl),
        time_balance: 0.0,
        frame_balance: 0,
        turn: 0,
        motion_queue: VecDeque::new(),
        needs_render: true,
    };
    let mut selected_pen: Box<dyn Pen> = Box::new(ElementPen { element: &SAND });
    let mut drawing = false;
    let mut last_mouse_pos = (-1.0, -1.0);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(args) = e.button_args() {
            match args.button {
                Button::Mouse(MouseButton::Left) => {
                    match args.state {
                        ButtonState::Press => {
                            drawing = true;
                            selected_pen.draw(&mut app.world, last_mouse_pos.0, last_mouse_pos.1);
                            //draw(&mut app.world, selected_element, last_mouse_pos.0, last_mouse_pos.1);
                        }
                        ButtonState::Release => {
                            drawing = false;
                        }
                    }
                }
                Button::Keyboard(key) => {
                    let element = match key {
                        Key::D1 => Some(&SAND),
                        Key::D2 => Some(&FIRE),
                        Key::D3 => Some(&GAS),
                        Key::D4 => Some(&ROCK),
                        Key::D5 => Some(&WATER),
                        // Key::D6 => Some(&WALL),
                        _ => None, // Key not recognized, do nothing
                    };
                    if let Some(element) = element {
                        selected_pen = Box::new(ElementPen { element });
                    }
                }
                _ => {}
            }
        }

        if let Some(args) = e.mouse_cursor_args() {
            if drawing {
                selected_pen.draw(&mut app.world, args[0], args[1]);
            } else {
                last_mouse_pos = (args[0], args[1]);
            }
        }
    }
}
