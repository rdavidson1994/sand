extern crate graphics;

use crate::{Color, SetupList, ElementId, ElementPen};
use graphics::Context as GraphicsContext;
use opengl_graphics::GlGraphics; // ugh

const SELECTION_HIGHLIGHT : Color = [0.8, 0.8, 0.1, 1.0];
const BUTTON_WIDTH : f64 = 30.0;
const BUTTON_HEIGHT : f64 = 10.0;
const PEN_BUTTON_SIZE : f64 = 10.0;
const BUTTON_PADDING_X : f64 = 5.0;
const BUTTON_PADDING_Y : f64 = 5.0;
const PEN_SIZES : [usize; 5] = [1, 2, 3, 4, 5];

struct PenSizeButton {
    pen_size: usize,
    selected: bool,
    upper_left: (f64, f64)
}

impl PenSizeButton {
    fn contains(&self, x: f64, y: f64) -> bool {
        self.upper_left.0 < x
        && self.upper_left.0 + PEN_BUTTON_SIZE > x
        && self.upper_left.1 < y
        && self.upper_left.1 + PEN_BUTTON_SIZE > y
    }

    fn outer_edge_width(&self) -> f64 {
        PEN_BUTTON_SIZE + 1.0 - 2.0*self.pen_size as f64
    }

    fn draw(&self, context: GraphicsContext, gl_graphics: &mut GlGraphics) {
        if self.selected {
            let selection_rectangle = [
                self.upper_left.0 - BUTTON_PADDING_X,
                self.upper_left.1 - BUTTON_PADDING_Y,
                PEN_BUTTON_SIZE + 2.0*BUTTON_PADDING_X,
                PEN_BUTTON_SIZE + 2.0*BUTTON_PADDING_Y,
            ];

            graphics::rectangle(
                SELECTION_HIGHLIGHT,
                selection_rectangle,
                context.transform,
                gl_graphics
            )
        }
        let rectangle = [
            self.upper_left.0,
            self.upper_left.1,
            self.pen_size as f64 * 2.0 - 1.0,
            self.pen_size as f64 * 2.0 - 1.0,
        ];
        graphics::rectangle(
            [1.0, 0.0, 1.0, 1.0],
            rectangle,
            context.transform,
            gl_graphics
        )
    }
}

struct ElementButton {
    color: Color,
    element_id: ElementId,
    selected: bool,
    upper_left: (f64, f64),
}

impl ElementButton {
    fn contains(&self, x: f64, y: f64) -> bool {
        self.upper_left.0 < x
        && self.upper_left.0 + BUTTON_WIDTH > x
        && self.upper_left.1 < y
        && self.upper_left.1 + BUTTON_HEIGHT > y
    }

    fn draw(&self, context: GraphicsContext, gl_graphics: &mut GlGraphics) {
        if self.selected {
            let selection_rectangle = [
                self.upper_left.0 - BUTTON_PADDING_X,
                self.upper_left.1 - BUTTON_PADDING_Y,
                BUTTON_WIDTH + 2.0*BUTTON_PADDING_X,
                BUTTON_HEIGHT + 2.0*BUTTON_PADDING_Y,
            ];

            graphics::rectangle(
                SELECTION_HIGHLIGHT,
                selection_rectangle,
                context.transform,
                gl_graphics
            )
        }
        let rectangle = [
            self.upper_left.0,
            self.upper_left.1,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
        ];
        graphics::rectangle(
            self.color,
            rectangle,
            context.transform,
            gl_graphics
        )
    }
}

pub struct ElementMenu {
    element_buttons : Vec<ElementButton>,
    selected_element_index : usize,
    pen_size_buttons : Vec<PenSizeButton>,
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
            let button = ElementButton {
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

        let pen_button_stride = PEN_BUTTON_SIZE + 2.0 * BUTTON_PADDING_X;
        let pen_buttons_left = buttons_per_row as f64 
            * (BUTTON_WIDTH + 2.0 * BUTTON_PADDING_X)
            + BUTTON_PADDING_X;

        let mut pen_buttons = vec![];
        for size in PEN_SIZES.iter() {
            let pen_button = PenSizeButton {
                pen_size: *size,
                selected: if *size == 1 { true } else { false },
                upper_left: (
                    pen_buttons_left + ((size-1) as f64) * pen_button_stride,
                    BUTTON_PADDING_Y,
                )
            };
            pen_buttons.push(pen_button);
        }

        ElementMenu { 
            pen_size_buttons: pen_buttons,
            element_buttons: buttons,
            selected_element_index: 0,
            selected_pen_size: 1,
        }
    }
    
    pub fn draw(
        &self, 
        context: GraphicsContext,
        gl_graphics: &mut GlGraphics,
    ) {
        for button in &self.element_buttons {
            button.draw(context, gl_graphics);
        }

        for button in &self.pen_size_buttons {
            button.draw(context, gl_graphics);
        }
    }

    pub fn build_pen(&self) -> ElementPen {
        ElementPen { 
            element: self.element_buttons[self.selected_element_index].element_id.get_element(),
            radius: self.selected_pen_size as i32,
        }
    }

    pub fn on_click(&mut self, x: f64, y: f64) -> Option<ElementPen> {
        let mut clicked_index = None;
        for (i, button) in self.element_buttons.iter().enumerate() {
            if button.contains(x, y) {
                clicked_index = Some(i);
                break;
            }
        }

        if let Some(i) = clicked_index {
            self.element_buttons[self.selected_element_index].selected = false;
            self.element_buttons[i].selected = true;
            self.selected_element_index = i;
            return Some(self.build_pen());
        }

        let mut clicked_index = None;
        for (i, button) in self.pen_size_buttons.iter().enumerate() {
            if button.contains(x,y) {
                clicked_index = Some(i);
            }
        }

        if let Some(i) = clicked_index {
            for button in &mut self.pen_size_buttons {
                if button.pen_size == self.selected_pen_size {
                    button.selected = false;
                }
            }
            self.pen_size_buttons[i].selected = true;
            self.selected_pen_size = PEN_SIZES[i];
            return Some(self.build_pen());
        }
        return None;
    }
}