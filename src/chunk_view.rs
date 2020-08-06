use crate::element::{FIXED, FLUID};
use crate::tile::Tile;
use crate::world::{CollisionSideEffectTable, PairwiseMutate};
use crate::{above, adjacent_x, neighbors, raw_neighbors, WORLD_WIDTH};
use rand::Rng;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug)]
pub struct ChunkIndex(usize);

impl ChunkIndex {
    pub fn new(index: usize) -> ChunkIndex {
        ChunkIndex(index)
    }
}

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

pub struct CollisionChunkView<'a, T> {
    chunk: &'a mut [T],
    /// Index of whichever particle has lower element id
    first_index: usize,
    /// Index of whichever particle has higher element id
    second_index: usize,
}

impl<'a, T> CollisionChunkView<'a, T> {
    pub fn new(chunk: &'a mut [T], first_index: usize, second_index: usize) -> Self {
        CollisionChunkView {
            chunk,
            first_index,
            second_index,
        }
    }

    /// A chunk view for the first particle
    pub fn first(&'a mut self) -> ChunkView<'a, T> {
        ChunkView::new(self.chunk, self.first_index)
    }

    /// A chunk view for the second particle
    pub fn second(&'a mut self) -> ChunkView<'a, T> {
        ChunkView::new(self.chunk, self.second_index)
    }

    /// Applies the given function to all neighboring indexes of the first particle,
    /// excluding the first and second particles themselves.
    pub fn for_neighbors_of_first(&mut self, mut f: impl FnMut(&mut T)) {
        for i in raw_neighbors(self.first_index) {
            if i != self.second_index {
                f(&mut self.chunk[i]);
            }
        }
    }

    /// Applies the given function to all neighboring indexes of the second particle,
    /// excluding the first and second particles themselves.
    pub fn for_neighbors_of_second(&mut self, mut f: impl FnMut(&mut T)) {
        for i in raw_neighbors(self.second_index) {
            if i != self.first_index {
                f(&mut self.chunk[i]);
            }
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

impl<'a, T> Index<ChunkIndex> for CollisionChunkView<'a, T> {
    type Output = T;
    fn index(&self, index: ChunkIndex) -> &Self::Output {
        &self.chunk[index.0]
    }
}

impl<'a, T> IndexMut<ChunkIndex> for CollisionChunkView<'a, T> {
    fn index_mut(&mut self, index: ChunkIndex) -> &mut Self::Output {
        &mut self.chunk[index.0]
    }
}

pub struct Chunk<'a> {
    slice: &'a mut [Option<Tile>],
    side_effects: &'a CollisionSideEffectTable,
}

impl<'a> Chunk<'a> {
    pub fn new<'r>(
        slice: &'r mut [Option<Tile>],
        side_effects: &'r HashMap<
            (u8, u8),
            fn(Tile, Tile, CollisionChunkView<Option<Tile>>) -> (Option<Tile>, Option<Tile>),
        >,
    ) -> Chunk<'r> {
        Chunk {
            slice,
            side_effects,
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.slice.swap(i, j);
        if let Some(above_source) = above(i) {
            if let Some(tile) = &mut self.slice[above_source] {
                tile.paused = false;
            }
        }
    }

    pub fn move_particle(&mut self, source: ChunkIndex, delta_x: i8, delta_y: i8) {
        //destination = source.0 + delta_x + WORLD_WIDTH * delta_y;
        let source = source.0;
        let mut destination = source;
        if delta_x < 0 {
            destination -= 1;
        } else if delta_x > 0 {
            destination += 1;
        }
        if delta_y < 0 {
            destination -= WORLD_WIDTH as usize;
        } else if delta_y > 0 {
            destination += WORLD_WIDTH as usize;
        }
        let (source_tile, destination_tile) = self.slice.mutate_pair(source, destination);
        match (source_tile, destination_tile) {
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
                self.trigger_collision_side_effects(ChunkIndex(source), ChunkIndex(destination));
            }
        }
    }

    pub fn trigger_collision_side_effects(
        &mut self,
        source: ChunkIndex,
        destination: ChunkIndex,
    ) -> bool {
        // If we can't unwrap here, a collision occurred in empty space
        let source_tile = self[source].unwrap();
        let destination_tile = self[destination].unwrap();
        let source_element_id = source_tile.element_id();
        let destination_element_id = destination_tile.element_id();
        let first_element_id = std::cmp::min(source_element_id, destination_element_id);
        let last_element_id = std::cmp::max(source_element_id, destination_element_id);
        if let Some(reaction) = self.side_effects.get(&(first_element_id, last_element_id)) {
            if first_element_id == source_element_id {
                let (source_after, destination_after) = reaction(
                    source_tile,
                    destination_tile,
                    CollisionChunkView::new(self.slice, source.0, destination.0),
                );
                self[source] = source_after;
                self[destination] = destination_after;
            } else {
                let (destination_after, source_after) = reaction(
                    destination_tile,
                    source_tile,
                    CollisionChunkView::new(self.slice, destination.0, source.0),
                );
                self[source] = source_after;
                self[destination] = destination_after;
            }
            true
        } else {
            false
        }
    }

    pub fn create_view(&mut self, index: ChunkIndex) -> ChunkView<Option<Tile>> {
        ChunkView::new(self.slice, index.0)
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
