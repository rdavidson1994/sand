// #[derive(Clone, PartialEq, Eq)]
// pub enum ElementState {
//     None,
//     MetalState(MetalState),
//     ActiveFire
// }

// #[derive(Clone, PartialEq, Eq)]
// pub enum MetalState {
//     Neutral,
//     ChargedHead,
//     ChargedTail,
// }
use crate::{ElementId, SpecialElementInfo};
use std::num::NonZeroU8;

#[derive(Clone)]
pub struct ElementState {
    element_id: ElementId,
    special_info: SpecialElementInfo,
}

impl ElementState {
    pub fn new(element_id: ElementId) -> ElementState {
        ElementState {
            element_id,
            special_info: SpecialElementInfo(unsafe {
                NonZeroU8::new(1).unwrap()
            }),
        }
    }
}

#[derive(Clone)]
pub struct ElementData {
    current: ElementState,
    staged: ElementState,
}

impl ElementData {
    pub fn new(state: ElementState) -> ElementData {
        ElementData {
            current: state.clone(),
            staged: state,
        }
    }

    pub fn element_id(&self) -> ElementId {
        self.current.element_id
    }

    #[allow(dead_code)]
    pub fn as_ref(&self) -> &ElementState {
        &self.current
    }

    pub fn stage(&mut self, element_state: ElementState) {
        self.staged = element_state;
    }

    pub fn commit(&mut self) {
        self.current = self.staged.clone();
    }
}
