use crate::element::{ElementId, SpecialElementInfo};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
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

#[derive(Clone, Copy)]
pub struct ElementData {
    current: ElementState,
    staged: ElementState,
}

impl ElementData {
    pub fn new(state: ElementState) -> ElementData {
        ElementData {
            current: state,
            staged: state,
        }
    }

    pub fn element_id(&self) -> ElementId {
        self.current.element_id
    }

    pub fn adjust(&mut self, delta: i16) {
        let old = self.staged.special_info.as_u8() as i16;
        let new_or_overflow = old.saturating_add(delta);
        let new = if new_or_overflow > 255i16 {
            255u8
        } else if new_or_overflow < 1i16 {
            1u8
        } else {
            new_or_overflow as u8
        };
        self.stage(ElementState {
            element_id: self.staged.element_id,
            special_info: SpecialElementInfo::new(new),
        })
    }

    #[allow(dead_code)]
    pub fn as_ref(&self) -> &ElementState {
        &self.current
    }

    pub fn stage(&mut self, element_state: ElementState) {
        self.staged = element_state;
    }

    pub fn commit(&mut self) {
        self.current = self.staged;
    }
}
