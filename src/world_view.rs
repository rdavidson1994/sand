use crate::element::{FIXED, FLUID};
use crate::{above, adjacent_x, neighbors, raw_neighbors, WORLD_WIDTH};
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug)]
pub struct NeighborhoodIndex(usize);

impl NeighborhoodIndex {
    pub fn new(index: usize) -> NeighborhoodIndex {
        NeighborhoodIndex(index)
    }
}

pub struct NeighborhoodView<'a, T> {
    slice: &'a mut [T],
    index: usize,
}

impl<'a, T> NeighborhoodView<'a, T> {
    pub fn new(slice: &'a mut [T], index: usize) -> Self {
        NeighborhoodView { slice, index }
    }

    pub fn neighbors(&self) -> impl Iterator<Item = NeighborhoodIndex> {
        raw_neighbors(self.index).map(|x| NeighborhoodIndex(x))
    }

    pub fn above(&mut self) -> &mut T {
        &mut self.slice[self.index - WORLD_WIDTH as usize]
    }

    pub fn below(&mut self) -> &mut T {
        &mut self.slice[self.index + WORLD_WIDTH as usize]
    }

    pub fn for_each_neighbor(&mut self, mut f: impl FnMut(&mut T)) {
        for i in raw_neighbors(self.index) {
            f(&mut self.slice[i])
        }
    }
}

pub struct CollisionView<'a, T> {
    slice: &'a mut [T],
    /// Index of whichever particle has lower element id
    first_index: usize,
    /// Index of whichever particle has higher element id
    second_index: usize,
}

impl<'a, T> CollisionView<'a, T> {
    pub fn new(slice: &'a mut [T], first_index: usize, second_index: usize) -> Self {
        CollisionView {
            slice,
            first_index,
            second_index,
        }
    }

    /// A neighborhood view for the first particle
    pub fn first(&mut self) -> NeighborhoodView<T> {
        NeighborhoodView::new(self.slice, self.first_index)
    }

    /// A neighborhood view for the second particle
    pub fn second(&mut self) -> NeighborhoodView<T> {
        NeighborhoodView::new(self.slice, self.second_index)
    }

    /// Applies the given function to all neighboring indexes of the first particle,
    /// excluding the first and second particles themselves.
    pub fn for_neighbors_of_first(&mut self, mut f: impl FnMut(&mut T)) {
        for i in raw_neighbors(self.first_index) {
            if i != self.second_index {
                f(&mut self.slice[i]);
            }
        }
    }

    /// Applies the given function to all neighboring indexes of the second particle,
    /// excluding the first and second particles themselves.
    pub fn for_neighbors_of_second(&mut self, mut f: impl FnMut(&mut T)) {
        for i in raw_neighbors(self.second_index) {
            if i != self.first_index {
                f(&mut self.slice[i]);
            }
        }
    }
}

impl<'a, T> Index<NeighborhoodIndex> for NeighborhoodView<'a, T> {
    type Output = T;
    fn index(&self, index: NeighborhoodIndex) -> &Self::Output {
        &self.slice[index.0]
    }
}

impl<'a, T> IndexMut<NeighborhoodIndex> for NeighborhoodView<'a, T> {
    fn index_mut(&mut self, index: NeighborhoodIndex) -> &mut Self::Output {
        &mut self.slice[index.0]
    }
}

impl<'a, T> Index<NeighborhoodIndex> for CollisionView<'a, T> {
    type Output = T;
    fn index(&self, index: NeighborhoodIndex) -> &Self::Output {
        &self.slice[index.0]
    }
}

impl<'a, T> IndexMut<NeighborhoodIndex> for CollisionView<'a, T> {
    fn index_mut(&mut self, index: NeighborhoodIndex) -> &mut Self::Output {
        &mut self.slice[index.0]
    }
}
