mod tile;
mod fire;
mod world;

use crate::fire::{FIRE, FireElementSetup};
use crate::tile::{Tile, Vector};
use itertools::iproduct;
use rand::{Rng, thread_rng};
use std::ops::{Index, IndexMut};
use std::collections::VecDeque;
const WORLD_WIDTH : i32 = 120;
const WORLD_HEIGHT: i32 = 120;
const WORLD_SIZE : i32 = WORLD_HEIGHT*WORLD_WIDTH;
const TILE_PIXELS : i32 = 3;
const WINDOW_PIXEL_WIDTH : i32 = WORLD_WIDTH * TILE_PIXELS;
const WINDOW_PIXEL_HEIGHT : i32 = WORLD_HEIGHT * TILE_PIXELS;
const LOGICAL_FRAMES_PER_DISPLAY_FRAME : i32 = 10;
const GRAVITY_PERIOD : i32 = 20;
const DECAY_REACTION_PERIOD : i32 = 100;
const PAUSE_VELOCITY : i8 = 3;
const SECONDS_PER_LOGICAL_FRAME : f64 = 1.0 / 1400.0; // Based on square = 1inch
//graphics imports
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{
    EventSettings,
    Events
};
use piston::input::{
    RenderArgs,
    RenderEvent,
    UpdateArgs,
    UpdateEvent, 
    Button,
    ButtonEvent,
    ButtonState,
    MouseButton,
    MouseCursorEvent,
    Key,
};
use piston::window::WindowSettings;

pub trait ElementSetup {
    fn register_reactions(&mut self, world: &mut World);
}

trait PairwiseMutate {
    type T;
    fn mutate_pair(&mut self, first: usize, second: usize) -> (&mut Self::T, &mut Self::T);
}

impl<U> PairwiseMutate for [U] {
    type T = U;
    fn mutate_pair(&mut self, first: usize, second: usize) -> (&mut Self::T, &mut Self::T) {
        let swapped = second < first;
        let minimum = if !swapped { first } else { second };
        let maximum = if !swapped { second } else { first };
        if minimum == maximum {
            panic!("Attempt to mutate a pair consisting of the same index twice.")
        }
        let (head, tail) = self.split_at_mut(minimum + 1);
        if !swapped {
            (&mut head[minimum], &mut tail[maximum - minimum - 1])
        }
        else {
            (&mut tail[maximum - minimum - 1], &mut head[minimum])
        }
    }
}

// Can't use bitflags crate at the moment, since we need FLAG1 | FLAG2 to be const
type EFlag = u8;
const NO_FLAGS : EFlag = 0;
const GRAVITY : EFlag = 1 << 0;
const FIXED : EFlag = 1 << 1;
const PAUSE_EXEMPT : EFlag = 1 << 2;

#[derive(Default)]
pub struct Element {
    flags : EFlag,
    // symbol_l: char,
    // symbol_r: char,
    color : [f32; 4],
    mass: i8,
    id: u32,
    decay_reaction: Option<fn(&mut World, usize)>
}

impl Element {
    fn has_flag(&self, flag: EFlag) -> bool {
        flag & self.flags != 0
    }
}

fn in_bounds(x: i32, y: i32) -> bool {
    x >= 0 && x < WORLD_WIDTH
    && y >= 0 && y < WORLD_HEIGHT
}

#[inline]
fn below(position: usize) -> Option<usize> {
    position.checked_add(WORLD_WIDTH as usize)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn above(position: usize) -> Option<usize> {
    position.checked_sub(WORLD_WIDTH as usize)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn left(position: usize) -> Option<usize> {
    position.checked_sub(1)
    .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn right(position: usize) -> Option<usize>{
    position.checked_add(1)
        .filter(|x| x < &(WORLD_SIZE as usize))
}

#[inline]
fn adjacent_x(position1: usize, position2: usize) -> bool {
    let is_left = match left(position1) {
        None => false,
        Some(x) => x == position2
    };
    let is_right = match right(position1) {
        None => false,
        Some(x) => x == position2
    };
    is_left || is_right
}

#[inline]
#[allow(dead_code)]
fn adjacent_y(position1: usize, position2: usize) -> bool {
    let is_above = match above(position1) {
        None => false,
        Some(x) => x == position2
    };
    let is_below = match below(position1) {
        None => false,
        Some(x) => x == position2
    };
    is_above || is_below
}

fn move_particle(source: usize, destination: usize, world: &mut World) {
    // TODO: Switch this all up to use world[i] instead borrowing
    let (source_tile, dest_tile) = world.mutate_pair(source, destination);
    match (source_tile, dest_tile) {
    //match (world[source].as_mut(), world[destination].as_mut()) {
        (None, None) | (None, Some(_)) => {
            //Source particle has moved for some other reason - nothing to do
        }
        (Some(_), None) => {
            world.swap(source, destination);
        }
        (Some(ref mut s), Some(ref mut d)) => {
            s.velocity.x;
            d.velocity.x;
            if adjacent_x(source, destination) {
                if d.has_flag(FIXED) {
                    s.reflect_velocity_x();
                }
                else {
                    s.elastic_collide_x(d);
                    world.unpause(destination);
                }
            }
            else /*if adjacent_y(source, destination)*/ {
                if d.has_flag(FIXED) {
                    s.reflect_velocity_y();
                }
                else {
                    s.elastic_collide_y(d);
                    world.unpause(destination);
                }
            }
            world.trigger_collision_side_effects(source, destination);
            world.trigger_collision_reactions(source, destination);
        }
    }
}

fn has_stable_floor(position: usize, world: &World) -> bool {
    match below(position) {
        Some(floor_position) => {
            match &world[floor_position] {
                Some(tile) => {
                    tile.has_flag(FIXED)
                    || tile.paused
                },
                None => false
            }
        },
        None => true // The bottom of the world counts as stable
    }
}

fn pause_particles(world: &mut World) {
    for i in 0..WORLD_SIZE as usize {
        match &world[i] {
            Some(tile) => {
                if tile.paused 
                    || tile.has_flag(PAUSE_EXEMPT)
                    || tile.velocity.x.abs() > PAUSE_VELOCITY
                    || tile.velocity.y.abs() > PAUSE_VELOCITY
                    || !has_stable_floor(i, world) {
                    continue;
                }
            }
            None => {
                continue;
            }
        }
        // Since we didn't continue to the next iteration, world[i] is not None
        let tile = world[i].as_mut().unwrap();
        tile.paused = true;
        tile.velocity.x = 0;
        tile.velocity.y = 0;
    }
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
                    let (new_grid_x, new_grid_y) = (
                        old_grid_x + delta_x as i32,
                        old_grid_y + delta_y as i32
                    );
                    if in_bounds(new_grid_x, new_grid_y) 
                    {
                        let swap_pair = (i, point(new_grid_x, new_grid_y));
                        // this logic is to allow "trains" of adjacent particles
                        // to travel smoothly and not knock each other
                        if delta_y < 0 || (delta_y == 0 && delta_x < 0) {
                            motion_queue.push_back(swap_pair);
                        }
                        else {
                            motion_queue.push_front(swap_pair);
                        }
                    }
                }
            }
        }
    }

    for (i,j) in motion_queue {
        assert!(in_bounds(coords(*i).0, coords(*i).1));
        assert!(in_bounds(coords(*j).0, coords(*j).1));
        move_particle(*i, *j, world);
    };

    needs_update
}

fn apply_gravity(world: &mut World) {
    for i in 0..WORLD_SIZE as usize {
        match &mut world[i] {
            Some(ref mut tile) => {
                if tile.has_flag(GRAVITY) && !tile.paused {
                    tile.velocity.y = tile.velocity.y.saturating_add(1);
                }
            }
            None => { }
        }
    }
}

fn apply_decay_reactions(world: &mut World) {
    for i in 0..WORLD_SIZE as usize {
        if let Some(tile) = &world[i] {
            if let Some(reaction) = tile.element.decay_reaction {
                reaction(world, i);
            }
        }
    }
}

fn coords(i: usize) -> (i32, i32) {
    ((i % (WORLD_WIDTH as usize)) as i32, (i / (WORLD_WIDTH as usize)) as i32)
}

fn point(x: i32 , y: i32) -> usize {
    (x+y*WORLD_WIDTH) as usize
}

static WALL : Element = Element {
    flags: FIXED,
    color: [1.0, 1.0, 1.0, 1.0],
    mass: 127,
    id: 0,
    decay_reaction: None,
};

static ROCK : Element = Element {
    flags: GRAVITY,
    color: [0.5, 0.5, 0.5, 1.0],
    mass: 50,
    id: 1,
    decay_reaction: None,
};

static SAND : Element = Element {
    flags: GRAVITY,
    color: [1.0, 1.0, 0.5, 1.0],
    mass: 10,
    id: 2,
    decay_reaction: None,
};

static GAS : Element = Element {
    flags: NO_FLAGS,
    color: [1.0, 0.5, 1.0, 1.0],
    mass: 3,
    id: 3,
    decay_reaction: None,
};

static WATER : Element = Element {
    flags: GRAVITY | PAUSE_EXEMPT,
    color: [0.0, 0.0, 1.0, 1.0],
    mass: 8,
    id: 6,
    decay_reaction: Some(|w, p| {
        // Water "jiggles" slightly
        let mut rng = thread_rng();
        let water_tile = w[p].as_mut().unwrap();
        water_tile.velocity.x += rng.gen_range(-3, 4);
        // if let Some(below_pos) = below(p) {
        //     if w[below_pos].is_some() {
        //         let (x,y) = coords(below_pos);
        //         let (first_choice, second_choice) = if rng.gen::<bool>() {
        //             (x+1, x-1)
        //         } else {
        //             (x+1, x-1)
        //         };
        //         if in_bounds(first_choice, y) && w[point(first_choice, y)].is_none() {
        //             w.swap(point(first_choice, y), p);
        //         }
        //         else if in_bounds(second_choice, y) && w[point(second_choice, y)].is_none() {
        //             w.swap(point(second_choice, y), p)
        //         }
        //     }
        // }
    }),
};

//type World = [Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize];//Vec<Option<Tile>>;c

type CollisionSideEffect = fn(&mut World, usize, usize);
type CollisionReaction = fn(&mut Tile, &mut Tile);

pub struct World {
    grid: [Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize],
    collision_side_effects: std::collections::HashMap<
        (u32, u32),
        CollisionSideEffect
    >,
    collision_reactions: std::collections::HashMap<
        (u32, u32),
        CollisionReaction
    >
}

impl World {
    fn swap(&mut self, i: usize, j: usize) {
        self.grid.swap(i, j);
    }

    fn register_collision_reaction(
        &mut self,
        element1: &Element,
        element2: &Element,
        reaction: fn(&mut Tile, &mut Tile),
    ) {
        let first_id = std::cmp::min(element1.id, element2.id);
        let second_id = std::cmp::max(element1.id, element2.id);
        let reagent_ids = (first_id, second_id);
        let conflict = self.collision_reactions.insert(reagent_ids, reaction);
        match conflict {
            Some(_) => {panic!("Attempt to register a duplicate reaction for {:?}", reagent_ids)},
            None => () // All good
        }
    }

    fn register_collision_side_effect(
        &mut self,
        element1: &Element,
        element2: &Element,
        side_effect: fn(&mut World, usize, usize),
    ) {
        let first_id = std::cmp::min(element1.id, element2.id);
        let second_id = std::cmp::max(element1.id, element2.id);
        let reagent_ids = (first_id, second_id);
        let conflict = self.collision_side_effects.insert(reagent_ids, side_effect);
        match conflict {
            Some(_) => {panic!("Attempt to register a duplicate reaction for {:?}", reagent_ids)},
            None => () // All good
        }
    }

    fn trigger_collision_side_effects(&mut self, source: usize, destination: usize) -> bool {
        // If we can't unwrap here, a collision occured in empty space
        let source_element_id = self[source].as_mut().unwrap().element.id;
        let destination_element_id = self[destination].as_mut().unwrap().element.id;
        let first_element_id = std::cmp::min(source_element_id, destination_element_id);
        let last_element_id = std::cmp::max(source_element_id, destination_element_id);
        if let Some(reaction) = self.collision_side_effects.get_mut(&(first_element_id, last_element_id)) {
            if first_element_id == source_element_id {
                reaction(self, destination, source);
            }
            else {
                reaction(self, source, destination);
            }
            true
        }
        else {
            false
        }
    }
   
    fn trigger_collision_reactions(&mut self, source: usize, destination: usize) -> bool {
        let source_element_id = self[source].as_ref().unwrap().element.id;
        let destination_element_id = self[destination].as_ref().unwrap().element.id;
        let first_element_id = std::cmp::min(source_element_id, destination_element_id);
        let last_element_id = std::cmp::max(source_element_id, destination_element_id);
        if let Some(reaction) = self.collision_reactions.get_mut(&(first_element_id, last_element_id)) {
            let (source_option, destination_option) = self.grid.mutate_pair(source, destination);
            let (source_tile, destination_tile) = (
                source_option.as_mut().unwrap(),
                destination_option.as_mut().unwrap()
            );
            if first_element_id == source_tile.element.id {
                reaction(destination_tile, source_tile);
            }
            else {
                reaction(source_tile, destination_tile);
            }
            true
        }
        else {
            false
        }
    }

    fn mutate_pair(&mut self, first: usize, second: usize) -> (&mut Option<Tile>, &mut Option<Tile>) {
        self.grid.mutate_pair(first, second)
    }

    fn unpause(&mut self, initial_position: usize) {
        let mut current_position = initial_position;
        loop {
            if let Some(ref mut tile) = self[current_position] {
                if tile.paused {
                    tile.paused = false;
                    if let Some(new_position) = above(current_position) {
                        current_position = new_position;
                        // glorified goto lol
                        continue;
                    }
                }
            }
            // if any condition fails, exit the loop
            break;
        }
    }
}

impl Index<usize> for World {
    type Output=Option<Tile>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.grid[i]
    }
}

impl IndexMut<usize> for World {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.grid[i]
    }
}

fn for_neighbors(index: usize, mut f: impl FnMut(usize)) {
        let x = index as i32 % WORLD_WIDTH;
        let y = index as i32 / WORLD_WIDTH;
        iproduct!(-1i32..1, -1i32..1) // consider all adjacent tuples
            .filter(|&tuple| tuple != (0,0)) // exclude same tile
            .map(|(dx,dy)| (x+dx, y+dy))
            .filter(|&(x,y)|  // exclude tiles outside world bounds
                in_bounds(x,y)
            ).map(|(x,y)| (x+y*WORLD_WIDTH) as usize) // calculate index
            .for_each(|minimum| f(minimum)); // apply input function
}

#[allow(dead_code)]
fn populate_world_bullet(world: &mut World) {
    world[point(10,10)] = Some(Tile {
        element: &GAS,
        position: Vector {x: 0, y: 0},
        velocity: Vector {x: 127, y:0},
        paused: false,
    })
}

fn populate_world_water_bubble(world: &mut World) {
    for x in 1..WORLD_WIDTH-1 {
        for y in WORLD_HEIGHT-20..WORLD_HEIGHT-1 {
            world[point(x,y)] = Some(
                Tile {
                    element: &SAND,
                    position : Vector {
                        x : 0,
                        y : 0,
                    },
                    velocity : Vector {
                        x : 0,
                        y : 0,
                    },
                    paused : true,
                }
            )
        }
    }

    for x in 20..WORLD_WIDTH-20 {
        for y in WORLD_HEIGHT-65..WORLD_HEIGHT-45 {
            world[point(x,y)] = Some(
                Tile {
                    element: &SAND,
                    position : Vector {
                        x : 0,
                        y : 0,
                    },
                    velocity : Vector {
                        x : 0,
                        y : 0,
                    },
                    paused : false,
                }
            )
        }
    }
}

#[allow(dead_code)]
fn populate_world_pileup(world: &mut World) {
    //let mut rng = thread_rng();
    for x in 5..10 {
        for y in 5..10 {
            world[point(x,y)] = Some(
                Tile {
                    element: &GAS,
                    position : Vector {
                        x : 0,
                        y : 0,
                    },
                    velocity : Vector {
                        x : 10,
                        y : 0,
                    },
                    paused : false,
                }
            )
        }
    }

    for x in 55..60 {
        for y in 5..10 {
            world[point(x,y)] = Some(
                Tile {
                    element: &GAS,
                    position : Vector {
                        x : 0,//rng.gen_range(-50,50),
                        y : 0,//rng.gen_range(-50,50),
                    },
                    velocity : Vector {
                        x : -10,
                        y : 0,//10,
                    },
                    paused : false,
                }
            )
        }
    }
}

fn create_walls(world: &mut World) {
    for i in 0..WORLD_WIDTH {
        world[point(i, 0)] = Some(Tile::stationary(&WALL));
        world[point(i, WORLD_HEIGHT-1)] = Some(Tile::stationary(&WALL));
    }
    for i in 0..WORLD_HEIGHT {
        world[point(0, i)] = Some(Tile::stationary(&WALL));
        world[point(WORLD_WIDTH-1, i)] = Some(Tile::stationary(&WALL));
    }
}

#[allow(dead_code)]
fn populate_world(world: &mut World) {
    let mut rng = thread_rng();
    for i in 0..45 {
        let x_offset = rng.gen_range(0,20);
        let y_offset = rng.gen_range(0,20);
        //let element = &SAND;
        let element = if i < 15 {
            &ROCK
        } else if i < 30 {
            &SAND
        } else {
            &FIRE
        };
        world[point(15+x_offset, 5+y_offset)] = Some(Tile{
            element,
            paused: false,
            position: Vector {
                x: 0,
                y: 0,
            },
            velocity: Vector {
                x: rng.gen_range(-1,1),
                y: rng.gen_range(-1,1),
            },
        })
    }
}

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
        // println!("FPS: {}", 1.0/args.ext_dt);
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
                    let square = rectangle::square((x*TILE_PIXELS) as f64, (y*TILE_PIXELS) as f64, TILE_PIXELS as f64);
                    rectangle(tile.element.color, square, transform, gl);
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
            pause_particles(&mut self.world);
            if self.turn % GRAVITY_PERIOD == 0 {
                apply_gravity(&mut self.world);
            }
            if self.turn % DECAY_REACTION_PERIOD == 0 {
                apply_decay_reactions(&mut self.world);
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
            world[point(x, y)] = Some(Tile{
                element : self.element,
                velocity : Vector {
                    x : thread_rng().gen_range(-20,21),
                    y : thread_rng().gen_range(-20,21),
                },
                position : Vector { x : 0, y : 0 },
                paused : false,
            })
        }
    }
}

pub fn game_loop() {
    let open_gl = OpenGL::V3_2;
    let size = [WINDOW_PIXEL_WIDTH as u32, WINDOW_PIXEL_HEIGHT as u32];
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Falling sand", size)
        .graphics_api(open_gl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    const EMPTY_TILE : Option<Tile> = None;
    let mut world = World { 
        grid: [EMPTY_TILE; (WORLD_HEIGHT * WORLD_WIDTH) as usize],
        collision_side_effects: std::collections::HashMap::new(),
        collision_reactions: std::collections::HashMap::new(),
    };
    //let mut i = 0;
    create_walls(&mut world);
    populate_world_water_bubble(&mut world);
    FireElementSetup.register_reactions(&mut world);

    let mut app = App {
        world,
        gl: GlGraphics::new(open_gl),
        time_balance: 0.0,
        frame_balance: 0,
        turn: 0,
        motion_queue: VecDeque::new(),
        needs_render: true,
    };
    let mut selected_pen : Box<dyn Pen> = Box::new(ElementPen { element: &SAND });
    let mut events = Events::new(EventSettings::new());
    let mut drawing = false;
    let mut last_mouse_pos = (-1.0, -1.0);
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
                            selected_pen.draw(
                                &mut app.world,
                                last_mouse_pos.0,
                                last_mouse_pos.1
                            );
                            //draw(&mut app.world, selected_element, last_mouse_pos.0, last_mouse_pos.1);
                        },
                        ButtonState::Release => {
                            drawing = false;
                        }
                    }
                },
                Button::Keyboard(key) => {
                    let element = match key {
                        Key::D1 => Some(&SAND),
                        Key::D2 => Some(&FIRE),
                        Key::D3 => Some(&GAS),
                        Key::D4 => Some(&ROCK),
                        Key::D5 => Some(&WATER),
                        // Key::D6 => &WALL,
                        _ => None // Key not recognized, do nothing
                    };
                    if let Some(element) = element {
                        selected_pen = Box::new(ElementPen { element });
                    }
                },
                _ => { }
            }
        }

        if let Some(args) = e.mouse_cursor_args() {
            if drawing {
                selected_pen.draw(&mut app.world, args[0], args[1]);
            }
            else {
                last_mouse_pos = (args[0], args[1]);
            }
        }
    }
}