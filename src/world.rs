use crate::chunk_view::{Chunk, ChunkIndex};
use crate::element::{Element, FIXED, FLUID, GRAVITY, PAUSE_EXEMPT};
use crate::tile::{ElementState, Tile};
use crate::{
    above, adjacent_x, neighbor_count, CollisionReaction, CollisionSideEffect, PAUSE_VELOCITY,
    WORLD_HEIGHT, WORLD_SIZE, WORLD_WIDTH,
};
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

const EMPTY_TILE: Option<Tile> = None;

pub type CollisionSideEffectTable = HashMap<(u8, u8), CollisionSideEffect>;
pub type CollisionReactionTable = HashMap<(u8, u8), CollisionReaction>;

pub struct World {
    grid: Box<[Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize]>,
    collision_side_effects: CollisionSideEffectTable,
    collision_reactions: CollisionReactionTable,
}

pub struct Neighborhood<'a, T> {
    before_slice: &'a mut [T],
    after_slice: &'a mut [T],
}

impl<'a, T> Neighborhood<'a, T> {
    fn new(before_slice: &'a mut [T], after_slice: &'a mut [T]) -> Neighborhood<'a, T> {
        Neighborhood {
            before_slice,
            after_slice,
        }
    }

    pub fn for_each(&mut self, action: impl FnMut(&mut T)) {
        self.for_each_impl(action, WORLD_WIDTH as usize)
    }

    fn for_each_impl(&mut self, mut action: impl FnMut(&mut T), width: usize) {
        let i = self.before_slice.len();
        action(&mut self.before_slice[i - width - 1]);
        action(&mut self.before_slice[i - width]);
        action(&mut self.before_slice[i - width + 1]);
        action(&mut self.before_slice[i - 1]);
        action(&mut self.after_slice[0]);
        action(&mut self.after_slice[width - 2]);
        action(&mut self.after_slice[width - 1]);
        action(&mut self.after_slice[width]);
    }
}

fn mutate_neighborhood<T>(slice: &mut [T], index: usize) -> (&mut T, Neighborhood<T>) {
    let (before, center_and_after) = slice.split_at_mut(index);
    let (center, after) = center_and_after.split_at_mut(1);
    (&mut center[0], Neighborhood::new(before, after))
}

pub trait PairwiseMutate {
    type T;
    fn mutate_pair(&mut self, first: usize, second: usize) -> (&mut Self::T, &mut Self::T);
}

impl<U> PairwiseMutate for [U] {
    type T = U;
    fn mutate_pair(&mut self, first: usize, second: usize) -> (&mut Self::T, &mut Self::T) {
        if first == second {
            panic!("Attempt to mutate a pair consisting of the same index twice.")
        }
        let swapped = second < first;
        let minimum = if !swapped { first } else { second };
        let maximum = if !swapped { second } else { first };
        let (head, tail) = self.split_at_mut(minimum + 1);
        if !swapped {
            (&mut head[minimum], &mut tail[maximum - minimum - 1])
        } else {
            (&mut tail[maximum - minimum - 1], &mut head[minimum])
        }
    }
}

impl IndexMut<usize> for World {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.grid[i]
    }
}

impl Index<usize> for World {
    type Output = Option<Tile>;
    fn index(&self, i: usize) -> &Self::Output {
        &self.grid[i]
    }
}

impl World {
    pub fn new() -> World {
        World {
            grid: Box::new([EMPTY_TILE; (WORLD_HEIGHT * WORLD_WIDTH) as usize]),
            collision_side_effects: std::collections::HashMap::new(),
            collision_reactions: std::collections::HashMap::new(),
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.grid.swap(i, j);
        if let Some(above_source) = above(i) {
            if let Some(tile) = &mut self[above_source] {
                tile.paused = false;
            }
        }
    }

    pub fn neighbor_count(&self, i: usize, predicate: impl Fn(&Tile) -> bool) -> usize {
        neighbor_count(i, |j| match &self[j] {
            None => false,
            Some(tile) => predicate(tile),
        })
    }

    pub fn state_at(&self, i: usize) -> Option<&ElementState> {
        self[i].as_ref().map(|x| x.get_state())
    }

    pub fn move_particle(&mut self, source: usize, destination: usize) {
        let (source_tile, dest_tile) = self.mutate_pair(source, destination);
        match (source_tile, dest_tile) {
            //match (world[source].as_mut(), world[destination].as_mut()) {
            (None, _) => {
                //Source particle has moved for some other reason - nothing to do
            }
            (Some(_), None) => {
                self.swap(source, destination);
            }
            (Some(ref mut s), Some(ref mut d)) => {
                if adjacent_x(source, destination) {
                    if d.has_flag(FIXED) {
                        s.reflect_velocity_x();
                    } else {
                        s.elastic_collide_x(d);
                    }
                } else
                /*if adjacent_y(source, destination)*/
                {
                    if d.has_flag(FIXED) {
                        s.reflect_velocity_y();
                    } else {
                        s.elastic_collide_y(d);
                    }
                }
                d.paused = false;
                if d.has_flag(FLUID) && rand::thread_rng().gen_range(0, 2) == 0 {
                    // Fluids don't collide, they just push through
                    self.swap(source, destination);
                }
            }
        }
    }

    pub fn has_stable_floor(&self, position: usize) -> bool {
        match crate::below(position) {
            Some(floor_position) => match &self[floor_position] {
                Some(tile) => tile.has_flag(FIXED) || tile.paused,
                None => false,
            },
            None => true, // The bottom of the world counts as stable
        }
    }

    pub fn pause_particles(&mut self) {
        for i in 0..WORLD_SIZE as usize {
            match &self[i] {
                Some(tile) => {
                    if tile.paused
                        || tile.has_flag(PAUSE_EXEMPT)
                        || tile.velocity.x.abs() > PAUSE_VELOCITY
                        || tile.velocity.y.abs() > PAUSE_VELOCITY
                        || !self.has_stable_floor(i)
                    {
                        continue;
                    }
                }
                None => {
                    continue;
                }
            }
            // Since we didn't continue to the next iteration, world[i] is not None
            let tile = self[i].as_mut().unwrap();
            tile.paused = true;
            tile.velocity.x = 0;
            tile.velocity.y = 0;
        }
    }

    pub fn apply_gravity(&mut self) {
        self.grid.par_iter_mut().for_each(|square| match square {
            Some(ref mut tile) => {
                if tile.has_flag(GRAVITY) && !tile.paused && !tile.has_flag(FIXED) {
                    tile.velocity.y = tile.velocity.y.saturating_add(1);
                }
            }
            None => {}
        })
    }

    /// Calls the function once for each tile between the initial and final wall segments
    /// The function receives a mutable chunk and an index into the chunk
    /// The chunk is guaranteed to contain enough tiles to access the given index's
    /// Moore neighborhood of radius 2 (i.e. up to 2 tiles away orthogonally or diagonally)
    pub fn chunked_for_each(&mut self, f: impl Fn(Chunk, ChunkIndex) + Sync + Send) {
        const CHUNK_MUTATE_HEIGHT: usize = 10;
        // Each thread will mutate 10 rows of the world at a time
        const CHUNK_HEIGHT: usize = CHUNK_MUTATE_HEIGHT * 2;
        // To give a "buffer" to ensure that the whole Moore neighborhood is inside the chunk,
        // We will actually borrow a chunk of twice that height.
        const CHUNK_SIZE: usize = CHUNK_HEIGHT * WORLD_WIDTH as usize;
        const CHUNK_MUTATE_SIZE: usize = CHUNK_MUTATE_HEIGHT * WORLD_WIDTH as usize - 4;
        // Why -4?
        // We *could* simply mutate a full CHUNK_MUTATE_HEIGHT * WORLD_WIDTH
        // but subtracting 4 leaves out a few wall tiles, and more importantly
        // preserves our guarantee about Moore neighborhoods for the final,
        // undersized chunk in the second para_chunks_mut call.
        const CHUNK_MUTATE_START: usize = (2 * WORLD_WIDTH + 2) as usize;
        // CHUNK_MUTATE_START is the first "safe" index in the chunk to touch.
        // Anything prior, and we don't have the point's full Moore neighborhood
        // contained inside the chunk
        const CHUNK_MUTATE_END: usize = CHUNK_MUTATE_START + CHUNK_MUTATE_SIZE;
        let collision_side_effects = &self.collision_side_effects;
        let collision_reactions = &self.collision_reactions;
        self.grid
            .par_chunks_exact_mut(CHUNK_SIZE)
            .for_each(|slice| {
                for i in CHUNK_MUTATE_START..CHUNK_MUTATE_END {
                    f(
                        Chunk::new(slice, collision_side_effects, collision_reactions),
                        ChunkIndex::new(i),
                    );
                }
            });
        // Calling the exact version of this method is important -
        // Otherwise we end up with a useless half-sized chunk at the end
        // which can't keep our guarantee about the Moore neighborhood

        let offset_grid = &mut self.grid[CHUNK_SIZE / 2..];
        // For the next run, offset by half the chunk size
        // Since the previous run covered half the tiles,
        // this run will catch all the leftovers
        offset_grid.par_chunks_mut(CHUNK_SIZE).for_each(|slice| {
            for i in CHUNK_MUTATE_START..CHUNK_MUTATE_END {
                f(
                    Chunk::new(slice, collision_side_effects, collision_reactions),
                    ChunkIndex::new(i),
                );
            }
        });
        // Note that we do *not* call the exact version this time.
        // There is still one undersized chunk left over, but it's
        // *just* the right size to keep our neighborhood guarantee,
        // while still mutating all non-wall tiles.
    }

    pub fn apply_periodic_reactions(&mut self) {
        self.chunked_for_each(|mut chunk, i| {
            if let Some(tile) = &mut chunk[i] {
                if let Some(reaction) = tile.get_element().periodic_reaction {
                    chunk[i] = reaction(*tile, chunk.create_view(i));
                }
            }
        });

        self.grid.par_iter_mut().for_each(|square| {
            if let Some(tile) = square {
                tile.save_state();
            }
        });
    }

    pub fn register_collision_reaction(
        &mut self,
        element1: &Element,
        element2: &Element,
        reaction: CollisionReaction,
    ) {
        let first_id = element1.id;
        let second_id = element2.id;
        if second_id < first_id {
            panic!(
                "Incorrect collision reaction registration for ids {} {}:\
                Ensure that elements are in ascending order or id",
                first_id, second_id
            )
        }
        let reagent_ids = (first_id, second_id);
        let conflict = self.collision_reactions.insert(reagent_ids, reaction);
        match conflict {
            Some(_) => panic!(
                "Attempt to register a duplicate reaction for {:?}",
                reagent_ids
            ),
            None => (), // All good
        }
    }

    pub fn add_collision_side_effect(
        &mut self,
        element1: &Element,
        element2: &Element,
        side_effect: CollisionSideEffect,
    ) {
        let first_id = element1.id;
        let second_id = element2.id;
        if second_id < first_id {
            panic!(
                "Incorrect collision reaction registration for ids {} {}:\
                Ensure that elements are in ascending order or id",
                first_id, second_id
            )
        }
        let reagent_ids = (first_id, second_id);
        let conflict = self.collision_side_effects.insert(reagent_ids, side_effect);
        match conflict {
            Some(_) => panic!(
                "Attempt to register a duplicate reaction for {:?}",
                reagent_ids
            ),
            None => (), // All good
        }
    }

    pub fn mutate_pair(
        &mut self,
        first: usize,
        second: usize,
    ) -> (&mut Option<Tile>, &mut Option<Tile>) {
        self.grid.mutate_pair(first, second)
    }

    // returns (center, neighbors)
    // panics if self[index] is None
    pub fn mutate_neighbors(&mut self, index: usize) -> (&mut Tile, Neighborhood<Option<Tile>>) {
        let (center, nhood) = mutate_neighborhood(&mut *self.grid, index);
        match center.as_mut() {
            Some(mut_ref_tile) => (mut_ref_tile, nhood),
            None => panic!("Attempted to mutate the neighbors of an empty square."),
        }
    }

    pub fn unpause(&mut self, initial_position: usize) {
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

#[test]
pub fn mutate_neighborhood_test() {
    let mut data = [
        0, 0, 0, 0, 0, // Row 0
        0, 0, 0, 0, 0, // Row 1
        0, 0, 0, 0, 0, // Row 2
        0, 0, 0, 0, 0, // Row 3
        0, 0, 0, 0, 0, // Row 4
    ];
    let (center, mut neighbors) = mutate_neighborhood(&mut data[..], 2 + 5 * 2);
    *center += 9;

    let mut index = 1;
    neighbors.for_each_impl(
        |val| {
            *val += index;
            index += 1;
        },
        5,
    );
    assert_eq!(
        data,
        [
            0, 0, 0, 0, 0, // Row 0
            0, 1, 2, 3, 0, // Row 1
            0, 4, 9, 5, 0, // Row 2
            0, 6, 7, 8, 0, // Row 3
            0, 0, 0, 0, 0, // Row 4
        ]
    );
}
