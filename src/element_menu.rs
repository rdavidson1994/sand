extern crate graphics;

use crate::{Color, SetupList, ElementId};
use graphics::Context as GraphicsContext;
use opengl_graphics::GlGraphics; // ugh

const SELECTION_HIGHLIGHT : Color = [0.8, 0.8, 0.1, 1.0];
const BUTTON_WIDTH : f64 = 30.0;
const BUTTON_HEIGHT : f64 = 10.0;
const BUTTON_PADDING_X : f64 = 5.0;
const BUTTON_PADDING_Y : f64 = 5.0;
const PEN_SIZES : [usize; 4] = [1, 2, 3, 4];

struct Button {
    color: Color,
    element_id: ElementId,
    selected: bool,
    upper_left: (f64, f64),
}

impl Button {
    fn contains(&self, x: f64, y: f64) -> bool {
        self.upper_left.0 < x
        && self.upper_left.0 + BUTTON_WIDTH > x
        && self.upper_left.1 < y
        && self.upper_left.1 + BUTTON_HEIGHT > y
    }
}

pub struct ElementMenu {
    element_buttons : Vec<Button>,
    selected_button_index : usize,
    selected_pen_size : usize,
}

impl ElementMenu {
    pub fn new(setup_list: &SetupList, buttons_per_row: usize) -> Self {
        let mut buttons = vec![];
        let mut x : f64 = BUTTON_PADDING_X;
        let mut y : f64 = BUTTON_PADDING_Y;
        for setup in setup_list {
            let color = setup.build_element().get_color(1).clone();
            let id = setup.get_id();
            let button = Button {
                color: color,
                element_id: id,
                selected: false,
                upper_left : (x,y),
            };
            buttons.push(button);
            if buttons.len() % buttons_per_row == 0 {
                x = BUTTON_PADDING_X;
                y += BUTTON_HEIGHT + 2.0*BUTTON_PADDING_Y;
            }
            else {
                x += BUTTON_WIDTH + 2.0*BUTTON_PADDING_X;
            }
        }

        ElementMenu { 
            element_buttons: buttons,
            selected_button_index: 0,
            selected_pen_size: 1,
        }
    }
    
    pub fn draw(
        &self, 
        context: GraphicsContext,
        gl_graphics: &mut GlGraphics,
    ) {
        for button in &self.element_buttons {
            if button.selected {
                let selection_rectangle = [
                    button.upper_left.0 - BUTTON_PADDING_X,
                    button.upper_left.1 - BUTTON_PADDING_Y,
                    BUTTON_WIDTH + 2.0*BUTTON_PADDING_X,
                    BUTTON_HEIGHT + 2.0*BUTTON_PADDING_Y,
                ];

                graphics::rectangle(
                    [0.8, 0.8, 0.1, 1.0], // Goldish
                    selection_rectangle,
                    context.transform,
                    gl_graphics
                )
            }
            let rectangle = [
                button.upper_left.0,
                button.upper_left.1,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
            ];
            graphics::rectangle(
                button.color,
                rectangle,
                context.transform,
                gl_graphics
            )
        }
    }

    pub fn on_click(&mut self, x: f64, y: f64) -> Option<ElementId> {
        let mut clicked_index = None;
        for (i, button) in self.element_buttons.iter().enumerate() {
            if button.contains(x, y) {
                clicked_index = Some(i);
                break;
            }
        }

        match clicked_index {
            Some(i) => {
                self.element_buttons[self.selected_button_index].selected = false;
                self.element_buttons[i].selected = true;
                self.selected_button_index = i;
                Some(self.element_buttons[i].element_id)
            }
            None => None
        }
    }
}