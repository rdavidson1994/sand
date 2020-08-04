use crate::element::{FIXED, FLUID};
use crate::tile::Tile;
use crate::world::PairwiseMutate;
use crate::{above, adjacent_x, neighbors, raw_neighbors, WORLD_WIDTH};
use rand::Rng;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
pub struct ChunkIndex(usize);

pub struct ChunkView<'a, T> {
    chunk: &'a mut [T],
    index: usize,
}

impl<'a, T> ChunkView<'a, T> {
    pub fn new(chunk: &'a mut [T], index: usize) -> Self {
        ChunkView { chunk, index }
    }

    pub fn neighbors(&self) -> impl Iterator<Item = ChunkIndex> {
        neighbors(self.index).map(|x| ChunkIndex(x))
    }

    pub fn above(&mut self) -> &mut T {
        &mut self.chunk[self.index - WORLD_WIDTH as usize]
    }

    pub fn below(&mut self) -> &mut T {
        &mut self.chunk[self.index + WORLD_WIDTH as usize]
    }

    pub fn for_each_neighbor(&mut self, mut f: impl FnMut(&mut T)) {
        for i in raw_neighbors(self.index) {
            f(&mut self.chunk[i])
        }
    }
}

impl<'a, T> Index<ChunkIndex> for ChunkView<'a, T> {
    type Output = T;

    fn index(&self, index: ChunkIndex) -> &Self::Output {
        &self.chunk[index.0]
    }
}

impl<'a, T> IndexMut<ChunkIndex> for ChunkView<'a, T> {
    fn index_mut(&mut self, index: ChunkIndex) -> &mut Self::Output {
        &mut self.chunk[index.0]
    }
}

pub struct Chunk<'a> {
    slice: &'a mut [Option<Tile>],
}

impl<'a> Chunk<'a> {
    pub fn new(slice: &mut [Option<Tile>]) -> Chunk {
        Chunk { slice }
    }
    pub fn swap(&mut self, i: usize, j: usize) {
        self.slice.swap(i, j);
        if let Some(above_source) = above(i) {
            if let Some(tile) = &mut self.slice[above_source] {
                tile.paused = false;
            }
        }
    }

    pub fn move_particle(&mut self, source: usize, destination: usize) {
        let (source_tile, dest_tile) = self.slice.mutate_pair(source, destination);
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
                // TODO: Reimplement collision reactions
                // self.trigger_collision_reactions(source, destination);
                // self.trigger_collision_side_effects(source, destination);
            }
        }
    }
}

impl<'a> Index<ChunkIndex> for Chunk<'a> {
    type Output = Option<Tile>;

    fn index(&self, index: ChunkIndex) -> &Self::Output {
        &self.slice[index.0]
    }
}

impl<'a> IndexMut<ChunkIndex> for Chunk<'a> {
    fn index_mut(&mut self, index: ChunkIndex) -> &mut Self::Output {
        &mut self.slice[index.0]
    }
}
