use crate::water::WATER;
use crate::{
    point, ElementState, Tile, Vector, World, FIRE, GAS, ROCK, SAND, WALL, WORLD_HEIGHT,
    WORLD_WIDTH,
};
use rand::{self, Rng};

#[allow(dead_code)]
fn populate_world_bullet(world: &mut World) {
    world[point(10, 10)] = Some(Tile::new(
        ElementState::default(GAS.id()),
        Vector { x: 0, y: 0 },
        Vector { x: 127, y: 0 },
        false,
    ))
}

#[allow(dead_code)]
pub fn populate_world_water_bubble(world: &mut World) {
    for x in 1..WORLD_WIDTH - 1 {
        for y in WORLD_HEIGHT - 20..WORLD_HEIGHT - 1 {
            world[point(x, y)] = Some(Tile::new(
                ElementState::default(SAND.id()),
                Vector { x: 0, y: 0 },
                Vector { x: 0, y: 0 },
                true,
            ))
        }
    }

    for x in 20..WORLD_WIDTH - 20 {
        for y in WORLD_HEIGHT - 65..WORLD_HEIGHT - 45 {
            world[point(x, y)] = Some(Tile::new(
                ElementState::default(SAND.id()),
                Vector { x: 0, y: 0 },
                Vector { x: 0, y: 0 },
                false,
            ))
        }
    }
}

#[allow(dead_code)]
pub fn populate_world_pileup(world: &mut World) {
    //let mut rng = thread_rng();
    for x in 5..10 {
        for y in 5..10 {
            world[point(x, y)] = Some(Tile::new(
                ElementState::default(GAS.id()),
                Vector { x: 0, y: 0 },
                Vector { x: 0, y: 0 },
                false,
            ))
        }
    }

    for x in 55..60 {
        for y in 5..10 {
            world[point(x, y)] = Some(Tile::new(
                ElementState::default(GAS.id()),
                Vector {
                    x: 0, //rng.gen_range(-50,50),
                    y: 0, //rng.gen_range(-50,50),
                },
                Vector {
                    x: -10,
                    y: 0, //10,
                },
                false,
            ))
        }
    }
}

pub fn create_walls(world: &mut World) {
    for i in 0..WORLD_WIDTH {
        world[point(i, 0)] = Some(Tile::stationary(ElementState::default(WALL.id())));
        world[point(i, WORLD_HEIGHT - 1)] =
            Some(Tile::stationary(ElementState::default(WALL.id())));
    }
    for i in 0..WORLD_HEIGHT {
        world[point(0, i)] = Some(Tile::stationary(ElementState::default(WALL.id())));
        world[point(WORLD_WIDTH - 1, i)] = Some(Tile::stationary(ElementState::default(WALL.id())));
    }
}

#[allow(dead_code)]
fn populate_world(world: &mut World) {
    let mut rng = rand::thread_rng();
    for i in 0..45 {
        let x_offset = rng.gen_range(0, 20);
        let y_offset = rng.gen_range(0, 20);
        //let element = &SAND;
        let element = if i < 15 {
            &ROCK
        } else if i < 30 {
            &SAND
        } else {
            &FIRE
        };
        world[point(15 + x_offset, 5 + y_offset)] = Some(Tile::new(
            ElementState::default(element.id()),
            Vector { x: 0, y: 0 },
            Vector {
                x: rng.gen_range(-1, 1),
                y: rng.gen_range(-1, 1),
            },
            false,
        ))
    }
}
