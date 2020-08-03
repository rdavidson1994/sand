use crate::{neighbors, raw_neighbors, WORLD_WIDTH};
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
