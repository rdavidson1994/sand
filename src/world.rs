use std::ops::{IndexMut, Index};
use crate::{CollisionSideEffect, CollisionReaction, WORLD_HEIGHT, WORLD_WIDTH, adjacent_x, FIXED, Element, above, WORLD_SIZE, PAUSE_EXEMPT, PAUSE_VELOCITY, GRAVITY};
use crate::tile::Tile;

const EMPTY_TILE : Option<Tile> = None;

pub struct World {
    grid: Box<[Option<Tile>; (WORLD_HEIGHT * WORLD_WIDTH) as usize]>,
    collision_side_effects: std::collections::HashMap<
        (u32, u32),
        CollisionSideEffect
    >,
    collision_reactions: std::collections::HashMap<
        (u32, u32),
        CollisionReaction
    >
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

impl IndexMut<usize> for World {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.grid[i]
    }
}

impl Index<usize> for World {
    type Output=Option<Tile>;
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

    pub fn move_particle(&mut self, source: usize, destination: usize) {
        let (source_tile, dest_tile) = self.mutate_pair(source, destination);
        match (source_tile, dest_tile) {
        //match (world[source].as_mut(), world[destination].as_mut()) {
            (None, None) | (None, Some(_)) => {
                //Source particle has moved for some other reason - nothing to do
            }
            (Some(_), None) => {
                self.swap(source, destination);
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
                        self.unpause(destination);
                    }
                }
                else /*if adjacent_y(source, destination)*/ {
                    if d.has_flag(FIXED) {
                        s.reflect_velocity_y();
                    }
                    else {
                        s.elastic_collide_y(d);
                        self.unpause(destination);
                    }
                }
                self.trigger_collision_side_effects(source, destination);
                self.trigger_collision_reactions(source, destination);
            }
        }
    }

    pub fn has_stable_floor(&self, position: usize) -> bool {
        match crate::below(position) {
            Some(floor_position) => {
                match &self[floor_position] {
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

    pub fn pause_particles(&mut self) {
        for i in 0..WORLD_SIZE as usize {
            match &self[i] {
                Some(tile) => {
                    if tile.paused
                        || tile.has_flag(PAUSE_EXEMPT)
                        || tile.velocity.x.abs() > PAUSE_VELOCITY
                        || tile.velocity.y.abs() > PAUSE_VELOCITY
                        || !self.has_stable_floor(i) {
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
        for i in 0..WORLD_SIZE as usize {
            match &mut self[i] {
                Some(ref mut tile) => {
                    if tile.has_flag(GRAVITY) && !tile.paused {
                        tile.velocity.y = tile.velocity.y.saturating_add(1);
                    }
                }
                None => { }
            }
        }
    }

    pub fn apply_decay_reactions(&mut self) {
        for i in 0..WORLD_SIZE as usize {
            if let Some(tile) = &self[i] {
                if let Some(reaction) = tile.get_element().decay_reaction {
                    reaction(self, i);
                }
            }
        }
        // for i in 0..WORLD_SIZE as usize {
        //     if let Some(tile) = &mut self[i] {
        //         tile.save_state();
        //     }
        // }
    }

    pub fn register_collision_reaction(
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

    pub fn register_collision_side_effect(
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
        let source_element_id = self[source].as_ref().unwrap().get_element().id;
        let destination_element_id = self[destination].as_ref().unwrap().get_element().id;
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
        let source_element_id = self[source].as_ref().unwrap().get_element().id;
        let destination_element_id = self[destination].as_ref().unwrap().get_element().id;
        let first_element_id = std::cmp::min(source_element_id, destination_element_id);
        let last_element_id = std::cmp::max(source_element_id, destination_element_id);
        if let Some(reaction) = self.collision_reactions.get_mut(&(first_element_id, last_element_id)) {
            let (source_option, destination_option) = self.grid.mutate_pair(source, destination);
            let (source_tile, destination_tile) = (
                source_option.as_mut().unwrap(),
                destination_option.as_mut().unwrap()
            );
            if first_element_id == source_element_id {
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

