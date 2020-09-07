use crate::element::{Element, PeriodicReaction, FIXED, FLUID, GRAVITY, PAUSE_EXEMPT};
use crate::tile::{ElementState, Tile};
use crate::world_view::{CollisionView, NeighborhoodView};
use crate::{adjacent_x, neighbor_count, PAUSE_VELOCITY, WORLD_HEIGHT, WORLD_SIZE, WORLD_WIDTH};
use rand::Rng;
use std::ops::{Index, IndexMut};

const EMPTY_TILE: Option<Tile> = None;

type Grid = [Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize];
type CollisionSideEffect =
    fn(Tile, Tile, CollisionView<Option<Tile>>) -> (Option<Tile>, Option<Tile>);
type CollisionReaction = fn(Tile, Tile) -> (Option<Tile>, Option<Tile>);

pub struct World {
    grid: Box<Grid>,
    collision_side_effects: std::collections::HashMap<(u8, u8), CollisionSideEffect>,
    collision_reactions: std::collections::HashMap<(u8, u8), CollisionReaction>,
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

pub(crate) trait PairwiseMutate {
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
                }
                /*if adjacent_y(source, destination)*/
                else if d.has_flag(FIXED) {
                    s.reflect_velocity_y();
                } else {
                    s.elastic_collide_y(d);
                }
                if d.has_flag(FLUID) && rand::thread_rng().gen_range(0, 2) == 0 {
                    // Fluids don't collide, they just push through
                    self.swap(source, destination);
                }
                //self.trigger_collision_reactions(source, destination);
                self.trigger_collision_side_effects(source, destination);
            }
        }
    }

    pub fn has_stable_floor(&self, position: usize) -> bool {
        match crate::below(position) {
            Some(floor_position) => match &self[floor_position] {
                Some(tile) => tile.has_flag(FIXED) || tile.velocity.is_zero(),
                None => false,
            },
            None => true, // The bottom of the world counts as stable
        }
    }

    pub fn pause_particles(&mut self) {
        for i in 0..WORLD_SIZE as usize {
            match &self[i] {
                None => {
                    continue;
                }
                Some(tile) => {
                    if
                    /*tile.paused
                    ||*/
                    tile.has_flag(PAUSE_EXEMPT)
                        || tile.velocity.x.abs() > PAUSE_VELOCITY
                        || tile.velocity.y.abs() > PAUSE_VELOCITY
                        || !self.has_stable_floor(i)
                    {
                        continue;
                    }
                }
            }
            // Since we didn't continue to the next iteration, world[i] is not None
            let tile = self[i].as_mut().unwrap();
            //tile.paused = true;
            tile.velocity.x = 0;
            tile.velocity.y = 0;
        }
    }

    pub fn apply_gravity(&mut self) {
        for i in 0..WORLD_SIZE as usize {
            if self.has_stable_floor(i) {
                continue;
            }
            match &mut self[i] {
                Some(ref mut tile) => {
                    if tile.has_flag(GRAVITY) && !tile.has_flag(FIXED) {
                        tile.velocity.y = tile.velocity.y.saturating_add(1);
                    }
                }
                None => {}
            }
        }
    }

    pub fn apply_periodic_reactions(&mut self) {
        for i in 0..WORLD_SIZE as usize {
            if let Some(tile) = self[i].clone() {
                match tile.get_element().periodic_reaction {
                    PeriodicReaction::Some(reaction) => {
                        self[i] = reaction(tile, NeighborhoodView::new(self.grid.as_mut(), i));
                    }
                    PeriodicReaction::None => {
                        // Do nothing
                    }
                    PeriodicReaction::DecayInto {
                        element_id,
                        lifetime,
                        rarity,
                    } => {
                        if rand::thread_rng().gen_range(0, rarity) == 0 {
                            let mut new_tile = tile.clone();
                            new_tile.edit_state(
                                tile.get_element().id(),
                                tile.special_info().saturating_add(1),
                            );
                            // Increase "temperature" by one
                            if new_tile.special_info() == lifetime {
                                // If we hit 255, melt
                                new_tile.set_element(element_id)
                            }
                            self[i] = Some(new_tile);
                        }
                    }
                    PeriodicReaction::DecayToNothing { lifetime, rarity } => {
                        if rand::thread_rng().gen_range(0, rarity) == 0 {
                            let mut new_tile = tile.clone();
                            new_tile.edit_state(
                                tile.get_element().id(),
                                tile.special_info().saturating_add(1),
                            );
                            // Increase "temperature" by one
                            if new_tile.special_info() == lifetime {
                                // If we hit 255, melt
                                self[i] = None
                            } else {
                                self[i] = Some(new_tile);
                            }
                        }
                    }
                }
            }
        }
        for i in 0..WORLD_SIZE as usize {
            if let Some(tile) = &mut self[i] {
                tile.save_state();
            }
        }
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
        if conflict.is_some() {
            panic!(
                "Attempt to register a duplicate reaction for {:?}",
                reagent_ids
            )
        }
    }

    pub fn register_collision_side_effect(
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
        if conflict.is_some() {
            panic!(
                "Attempt to register a duplicate reaction for {:?}",
                reagent_ids
            )
        }
    }

    pub fn trigger_collision_side_effects(&mut self, source: usize, destination: usize) -> bool {
        // If we can't unwrap here, a collision occurred in empty space
        let source_tile = self[source].clone().unwrap();
        let destination_tile = self[destination].clone().unwrap();
        let source_element_id = source_tile.element_id();
        let destination_element_id = destination_tile.element_id();
        let first_element_id = std::cmp::min(source_element_id, destination_element_id);
        let last_element_id = std::cmp::max(source_element_id, destination_element_id);
        let (first_tile, second_tile, first_index, second_index) =
            if source_element_id == first_element_id {
                (source_tile, destination_tile, source, destination)
            } else {
                (destination_tile, source_tile, destination, source)
            };

        if let Some(reaction) = self
            .collision_side_effects // rustfmt-skip
            .get(&(first_element_id, last_element_id))
        {
            let (mut first_after, mut second_after) = reaction(
                first_tile,
                second_tile,
                CollisionView::new(self.grid.as_mut(), first_index, second_index),
            );
            // Because the public methods on tiles edit the staged state,
            // We have to save states here.
            // We can't assume the periodic reaction loop will handle it for us.
            if let Some(ref mut first_after) = first_after {
                first_after.save_state();
            }
            if let Some(ref mut second_after) = second_after {
                second_after.save_state();
            }
            self[first_index] = first_after;
            self[second_index] = second_after;
            return true;
        }
        if let Some(reaction) = self
            .collision_reactions // rustfmt-skip
            .get(&(first_element_id, last_element_id))
        {
            let (mut first_after, mut second_after) = reaction(first_tile, second_tile);
            // Because the public methods on tiles edit the staged state,
            // We have to save states here.
            // We can't assume the periodic reaction loop will handle it for us.
            if let Some(ref mut first_after) = first_after {
                first_after.save_state();
            }
            if let Some(ref mut second_after) = second_after {
                second_after.save_state();
            }
            self[first_index] = first_after;
            self[second_index] = second_after;
            return true;
        }
        false
    }

    pub(crate) fn mutate_pair(
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
