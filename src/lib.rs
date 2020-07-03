//use itertools::iproduct;
use bitflags::bitflags;
use crossterm::{QueueableCommand, cursor};
use std::io::{Write, stdout, Stdout};
use rand::{Rng, thread_rng};
const WORLD_WIDTH : i32 = 40;
const WORLD_HEIGHT: i32 = 40;
const WORLD_SIZE : i32 = WORLD_HEIGHT*WORLD_WIDTH;
const DISPLAY_PERIOD : i32 = 1;
const GRAVITY_PERIOD : i32 = 20;


bitflags! {
    #[derive(Default)]
    struct ElementFlags : u8 {
        const NONE = 0b00000000;
        const GRAVITY = 0b00000001;
    }
}

#[derive(Default)]
struct Element {
    flags : ElementFlags,
    symbol: char,
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

#[derive(Clone)]
struct Tile {
    paused: bool,
    velocity: Vector,
    position: Vector,
    element: &'static Element,
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


// fn for_neighbors(index: i32, mut f: impl FnMut(usize)) {
//         let x = index % WORLD_WIDTH;
//         let y = index / WORLD_WIDTH;
//         iproduct!(-1i32..1, -1i32..1) // consider all adjacent tuples
//             .filter(|&tuple| tuple != (0,0)) // exclude same tile
//             .map(|(dx,dy)| (x+dx, y+dy))
//             .filter(|&(x,y)|  // exclude tiles outside world bounds
//                 in_bounds(x,y)
//             ).map(|(x,y)| (x+y*WORLD_WIDTH) as usize) // calculate index
//             .for_each(|minimum| f(minimum)); // apply input function
// }

#[inline]
fn below(position: usize) -> usize {
    return position + WORLD_WIDTH as usize;
}

#[inline]
fn above(position: usize) -> usize {
    return position - WORLD_WIDTH as usize;
}

#[inline]
fn left(position: usize) -> usize {
    return position - 1;
}

#[inline]
fn right(position: usize) -> usize{
    return position + 1;
}

#[inline]
fn adjacent_x(position1: usize, position2: usize) -> bool {
    position1 == left(position2) || position1 == right(position2)
}

#[inline]
fn adjacent_y(position1: usize, position2: usize) -> bool {
    position1 == above(position2) || position1 == below(position2)
}

fn mutate_pair(data: &mut World, first: usize, second: usize) -> (&mut Option<Tile>, &mut Option<Tile>) {
    let swapped = second < first;
    let minimum = if !swapped { first } else { second };
    let maximum = if !swapped { second } else { first };
    if minimum == maximum {
        panic!("Attempt to mutate a 'pair' consisting of the same tile twice.")
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
    // TODO: Handle collisions decently
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
                ellastic_collide_x(s, d)
            }
            if adjacent_y(source, destination) {
                ellastic_collide_y(s, d)
            }
        }
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


fn ellastic_collide_y(particle1: &mut Tile, particle2: &mut Tile) {
    let v1 = particle1.velocity.y as i32;
    let v2 = particle2.velocity.y as i32;
    let m1 = particle1.element.mass as i32;
    let m2 = particle2.element.mass as i32;
    particle1.velocity.y = ((((m1 - m2)/(m1 + m2))*v1 + 2*m2/(m1+m2))*v2) as i8;
    particle2.velocity.y = ((((m2 - m1)/(m2 + m1))*v2 + 2*m1/(m2+m1))*v1) as i8;
}

fn ellastic_collide_x(particle1: &mut Tile, particle2: &mut Tile) {
    let v1 = particle1.velocity.x as i32;
    let v2 = particle2.velocity.x as i32;
    let m1 = particle1.element.mass as i32;
    let m2 = particle2.element.mass as i32;
    particle1.velocity.x = ((((m1 - m2)/(m1 + m2))*v1 + 2*m2/(m1+m2))*v2) as i8;
    particle2.velocity.x = ((((m2 - m1)/(m2 + m1))*v2 + 2*m1/(m2+m1))*v1) as i8;
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

fn write_char(input: char, out: &mut Stdout) {
    out.write(&[input as u8]).unwrap();
}

fn display(world: &World) {
    let mut stdout = stdout();
    stdout.queue(cursor::SavePosition).unwrap();
    for i in 0..WORLD_SIZE as usize {
        if i % WORLD_WIDTH as usize == 0 {
            write_char('\n', &mut stdout);
        }
        match &world[i] {
            Some(tile) => {
                write_char(tile.element.symbol, &mut stdout);
                //stdout.write(format!("{}", tile.element.symbol).as_bytes()).unwrap();
                //print!("{}", tile.element.symbol);
            }
            None => {
                write_char(' ', &mut stdout);
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

type World = [Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize];//Vec<Option<Tile>>;

pub fn game_loop() {
    const EMPTY_TILE : Option<Tile> = None;
    let mut world = [EMPTY_TILE; (WORLD_HEIGHT * WORLD_WIDTH) as usize];

    static WALL : Element = Element {
        symbol: '#',
        flags: ElementFlags::NONE,
        mass: 127,
    };

    static ROCK : Element = Element {
        symbol: '*',
        flags: ElementFlags::GRAVITY,
        mass: 50
    };

    static SAND : Element = Element {
        symbol: '.',
        flags: ElementFlags::GRAVITY,
        mass: 10
    };

    let mut rng = thread_rng();
    for i in 0..WORLD_WIDTH {
        world[point(i, 0)] = Some(stationary_tile(&WALL));
        world[point(i, WORLD_HEIGHT-1)] = Some(stationary_tile(&WALL));
    }
    for i in 0..WORLD_HEIGHT {
        world[point(0, i)] = Some(stationary_tile(&WALL));
        world[point(WORLD_WIDTH-1, i)] = Some(stationary_tile(&WALL));
    }
    for i in 0..200 {
        let x_offset = rng.gen_range(0,20);
        let y_offset = rng.gen_range(0,20);
        let element = if i > 20 {
            &ROCK
        } else {
            &SAND
        };
        world[point(15+x_offset, 5+y_offset)] = Some(Tile{
            element: element,
            paused: false,
            position: Vector {
                x: 0,
                y: 0,
            },
            velocity: Vector {
                x: rng.gen_range(-50,-10),
                y: rng.gen_range(-50,-10),
            },
        })
    }
    let mut i = 0;
    loop {
        if i % DISPLAY_PERIOD == 0 {
            display(&world);
        }
        if i % GRAVITY_PERIOD == 0 {
            apply_gravity(&mut world);
        }
        apply_velocity(&mut world);
        i += 1;
    }
}