#[derive(Clone, PartialEq, Eq)]
pub enum ElementState {
    None,
    MetalState(MetalState),
    ActiveFire
}

#[derive(Clone, PartialEq, Eq)]
pub enum MetalState {
    Neutral,
    ChargedHead,
    ChargedTail,
}

#[derive(Clone)]
pub struct ElementData {
    current: ElementState,
    staged: ElementState,
}

impl ElementData {
    pub fn none() -> ElementData {
        ElementData {
            current : ElementState::None,
            staged : ElementState::None,
        }
    }
    pub fn new(state: ElementState) -> ElementData {
        ElementData {
            current : state.clone(),
            staged : state
        }
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