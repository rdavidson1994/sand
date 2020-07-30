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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ElementState {
    pub element_id: ElementId,
    pub special_info: SpecialElementInfo,
}

impl ElementState {
    pub fn default(element_id: ElementId) -> ElementState {
        ElementState {
            element_id,
            special_info: SpecialElementInfo::none(),
        }
    }

    pub fn new(element_id: ElementId, special_info: u8) -> ElementState {
        ElementState {
            element_id,
            special_info: SpecialElementInfo::new(special_info),
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
