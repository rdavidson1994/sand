//use itertools::iproduct;
use bitflags::bitflags;
use crossterm::{QueueableCommand, cursor};
use std::io::{Write, stdout, Stdout};
use rand::{Rng, thread_rng};
use std::time::Instant;
use std::convert::TryFrom;
use std::any::type_name;
use std::fmt::Display;
use num::Bounded;
const WORLD_WIDTH : i32 = 40;
const WORLD_HEIGHT: i32 = 40;
const WORLD_SIZE : i32 = WORLD_HEIGHT*WORLD_WIDTH;
const DISPLAY_PERIOD : i32 = 20;
const GRAVITY_PERIOD : i32 = 20;
const BASE_RESTITUTION : f64 = 0.5;
const PAUSE_VELOCITY : i8 = 3;


bitflags! {
    #[derive(Default)]
    struct ElementFlags : u8 {
        const NONE = 0b00000000;
        const GRAVITY = 0b00000001;
        const FIXED = 0b00000010;
    }
}

#[derive(Default)]
struct Element {
    flags : ElementFlags,
    symbol_l: char,
    symbol_r: char,
    mass: i8,
}


#[derive(Clone)]
struct Vector {
    x : i8,
    y : i8,
}

// impl Vector {
//     fn new(x: i8, y: i8) -> Self {
//         Vector { x, y }
//     }
// }

fn elastic_collide(v1: i8, v2: i8, m1: i8, m2: i8) -> (i8, i8) {
    let v1 = v1 as f64;
    let v2 = v2 as f64;
    let m1 = m1 as f64;
    let m2 = m2 as f64;
    let new_v1 = ((m1 - m2)/(m1 + m2))*v1 + 2.0*m2/(m1+m2)*v2;
    let new_v2 = ((m2 - m1)/(m2 + m1))*v2 + 2.0*m1/(m2+m1)*v1;
    (
        clamp_convert::<i32, i8>(new_v1.trunc() as i32),
        clamp_convert::<i32, i8>(new_v2.trunc() as i32),
    )
}

#[derive(Clone)]
struct Tile {
    paused: bool,
    velocity: Vector,
    position: Vector,
    element: &'static Element,
}

impl Tile {
    fn elastic_collide_y(&mut self, particle2: &mut Tile) {
        let (v1y, v2y) = elastic_collide(
            self.velocity.y,
            particle2.velocity.y,
            self.element.mass,
            particle2.element.mass,
        );
        self.velocity.y = v1y;
        particle2.velocity.y = v2y;
    }

    fn elastic_collide_x(&mut self, particle2: &mut Tile) {
        let (v1x, v2x) = elastic_collide(
            self.velocity.y,
            particle2.velocity.y,
            self.element.mass,
            particle2.element.mass,
        );
        self.velocity.x = v1x;
        particle2.velocity.x = v2x;
    }

    fn reflect_velocity_x(&mut self) {
        self.velocity.x = (-(self.velocity.x as f64) * BASE_RESTITUTION).trunc() as i8;
    }

    fn reflect_velocity_y(&mut self) {
        self.velocity.y = (-(self.velocity.y as f64) * BASE_RESTITUTION).trunc() as i8;
    }
    
}

fn stationary_tile(element: &'static Element) -> Tile {
    Tile{
        element: element,
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


fn mutate_pair(data: &mut World, first: usize, second: usize) -> (&mut Option<Tile>, &mut Option<Tile>) {
    let swapped = second < first;
    let minimum = if !swapped { first } else { second };
    let maximum = if !swapped { second } else { first };
    if minimum == maximum {
        panic!("Attempt to mutate a pair consisting of the same index twice.")
    }
    let (head, tail) = data.split_at_mut(minimum + 1);
    if !swapped {
        (&mut head[minimum], &mut tail[maximum - minimum - 1])
    }
    else {
        (&mut tail[maximum - minimum - 1], &mut head[minimum])
    }
}

fn move_particle(source: usize, destination: usize, world: &mut World) {
    // TODO: Switch this all up to use world[i] instead borrowing
    let (source_tile, dest_tile) = mutate_pair(world, source, destination);
    match (source_tile, dest_tile) {
    //match (world[source].as_mut(), world[destination].as_mut()) {
        (None, None) | (None, Some(_)) => {
            //Source particle has moved for some other reason - nothing to do
        }
        (Some(_), None) => {
            world.swap(source, destination);
        }
        (Some(ref mut s), Some(ref mut d)) => {
            if adjacent_x(source, destination) {
                if d.element.flags.contains(ElementFlags::FIXED) {
                    s.reflect_velocity_x();
                }
                else {
                    s.elastic_collide_x(d);
                    // oh no. Mutate pair made this bad :(
                    unpause(world, destination);
                }
            }
            else /*if adjacent_y(source, destination)*/ {
                if d.element.flags.contains(ElementFlags::FIXED) {
                    s.reflect_velocity_y();
                }
                else {
                    s.elastic_collide_y(d);
                    unpause(world, destination);
                }
            }
        }
    }
}

fn has_stable_floor(position: usize, world: &World) -> bool {
    match below(position) {
        Some(floor_position) => {
            match &world[floor_position] {
                Some(tile) => {
                    tile.element.flags.contains(ElementFlags::FIXED)
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
                    || !tile.element.flags.contains(ElementFlags::GRAVITY)
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

fn apply_velocity(world: &mut World) {
    let mut swaps : Vec<(usize, usize)> = vec![];
    for i in 0..WORLD_SIZE as usize {
        if let Some(ref mut tile) = &mut world[i] {
            if !tile.paused {
                let (new_x, overflowed_x) = tile.position.x.overflowing_add(tile.velocity.x);
                let (new_y, overflowed_y) = tile.position.y.overflowing_add(tile.velocity.y);
                tile.position.x = new_x;
                tile.position.y = new_y;
                if overflowed_x || overflowed_y {
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
                        swaps.push((i, point(new_grid_x, new_grid_y)));
                    }
                    else
                    {
                        print!("");
                    }
                }
            }
        }
    }

    for (i,j) in swaps {
        assert!(in_bounds(coords(i).0, coords(i).1));
        assert!(in_bounds(coords(j).0, coords(j).1));
        move_particle(i, j, world);
    }
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
        //dbg!("CLAMP CONVERT MAX");
        V::max_value()
    }
    else if t < V::min_value().into() {
        //dbg!("CLAMP CONVERT MIN");
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

fn apply_gravity(world: &mut World) {
    for i in 0..WORLD_SIZE as usize {
        match &mut world[i] {
            Some(ref mut tile) => {
                if tile.element.flags.contains(ElementFlags::GRAVITY) && !tile.paused {
                    tile.velocity.y = tile.velocity.y.saturating_add(1);
                }
            }
            None => { }
        }
    }
}

fn write_char_twice(input: char, out: &mut Stdout) {
    for _ in 0..2 {
        write_char(input, out)
    }
}

fn write_char(input: char, out: &mut Stdout) {
    out.write(&[input as u8]).unwrap();
}

fn display(world: &World, timestr: &str) {
    let mut stdout = stdout();
    stdout.queue(cursor::SavePosition).unwrap();
    stdout.write(timestr.as_bytes()).unwrap();
    write_char('\n', &mut stdout);
    for i in 0..WORLD_SIZE as usize {
        if i % WORLD_WIDTH as usize == 0 {
            write_char('\n', &mut stdout);
        }
        match &world[i] {
            Some(tile) => {
                write_char(tile.element.symbol_l, &mut stdout);
                write_char(tile.element.symbol_r, &mut stdout);
                //stdout.write(format!("{}", tile.element.symbol).as_bytes()).unwrap();
                //print!("{}", tile.element.symbol);
            }
            None => {
                write_char_twice(' ', &mut stdout);
            }
        }
    }
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
}


fn coords(i: usize) -> (i32, i32) {
    ((i % (WORLD_WIDTH as usize)) as i32, (i / (WORLD_WIDTH as usize)) as i32)
}

fn point(x: i32 , y: i32) -> usize {
    (x+y*WORLD_WIDTH) as usize
}

static WALL : Element = Element {
    symbol_l: '#',
    symbol_r: '#',
    flags: ElementFlags::FIXED,
    mass: 127,
};

static ROCK : Element = Element {
    symbol_l: '[',
    symbol_r: ']',
    flags: ElementFlags::GRAVITY,
    mass: 50
};

static SAND : Element = Element {
    symbol_l: '(',
    symbol_r: ')',
    flags: ElementFlags::GRAVITY,
    mass: 10
};

static GAS : Element = Element {
    symbol_l: '~',
    symbol_r: '~',
    flags: ElementFlags::NONE,
    mass: 3
};

static HEAVY_GAS : Element = Element {
    symbol_l: '&',
    symbol_r: '&',
    flags: ElementFlags::NONE,
    mass: 6
};

type World = [Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize];//Vec<Option<Tile>>;

fn unpause(world: &mut World, initial_position: usize) {
    let mut current_position = initial_position;
    loop {
        if let Some(ref mut tile) = world[current_position] {
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

fn populate_world_pileup(world: &mut World) {
    let mut rng = thread_rng();
    for x in 5..10 {
        for y in 5..10 {
            world[point(x,y)] = Some(
                Tile {
                    element: &SAND,
                    position : Vector {
                        x : rng.gen_range(-50,50),
                        y : rng.gen_range(-50,50),
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

    for x in 30..35 {
        for y in 5..10 {
            world[point(x,y)] = Some(
                Tile {
                    element: &SAND,
                    position : Vector {
                        x : rng.gen_range(-50,50),
                        y : rng.gen_range(-50,50),
                    },
                    velocity : Vector {
                        x : -10,
                        y : 0,
                    },
                    paused : false,
                }
            )
        }
    }
}

fn create_walls(world: &mut World) {
    for i in 0..WORLD_WIDTH {
        world[point(i, 0)] = Some(stationary_tile(&WALL));
        world[point(i, WORLD_HEIGHT-1)] = Some(stationary_tile(&WALL));
    }
    for i in 0..WORLD_HEIGHT {
        world[point(0, i)] = Some(stationary_tile(&WALL));
        world[point(WORLD_WIDTH-1, i)] = Some(stationary_tile(&WALL));
    }
}

fn populate_world(world: &mut World) {
    let mut rng = thread_rng();
    for i in 0..45 {
        let x_offset = rng.gen_range(0,20);
        let y_offset = rng.gen_range(0,20);
        let element = &SAND;
        let element = if i < 15 {
            &ROCK
        } else if i < 30 {
            &SAND
        } else {
            &GAS
        };
        world[point(15+x_offset, 5+y_offset)] = Some(Tile{
            element: element,
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

pub fn game_loop() {
    const EMPTY_TILE : Option<Tile> = None;
    let mut world = [EMPTY_TILE; (WORLD_HEIGHT * WORLD_WIDTH) as usize];
    let mut i = 0;
    create_walls(&mut world);
    populate_world(&mut world);
    let mut now = Instant::now();
    loop {
        if i % DISPLAY_PERIOD == 0 {
            let timestr = format!("Computed in {} ms", now.elapsed().as_millis());
            display(&world, &timestr);
            now = Instant::now();
        }
        pause_particles(&mut world);
        if i % GRAVITY_PERIOD == 0 {
            apply_gravity(&mut world);
        }
        apply_velocity(&mut world);
        i += 1;
    }
}