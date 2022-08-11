use crate::{
    apply_velocity, coords, element_menu::ElementMenu, world::World, Pen, GRAVITY_PERIOD,
    PLAY_AREA_PIXEL_HEIGHT, REACTION_PERIOD, TILE_PIXELS, UPDATES_PER_FRAME, WORLD_SIZE,
};
use opengl_graphics::GlGraphics;
use piston::{Button, ButtonArgs, ButtonState, MouseButton, RenderArgs, UpdateArgs};
use std::collections::VecDeque;

pub struct App {
    gl: GlGraphics,
    turn: i32,
    world: World,
    element_menu: ElementMenu,
    motion_queue: VecDeque<(usize, usize)>,
    selected_pen: Box<dyn Pen>,
    drawing: bool,
    last_mouse_pos: (f64, f64),
}

impl App {
    pub fn new(
        gl: GlGraphics,
        world: World,
        element_menu: ElementMenu,
        selected_pen: Box<dyn Pen>,
    ) -> Self {
        Self {
            gl,
            turn: 0,
            world,
            element_menu,
            motion_queue: Default::default(),
            selected_pen,
            drawing: false,
            last_mouse_pos: (-1.0, -1.0),
        }
    }
    pub fn render(&mut self, args: &RenderArgs) {
        // let fps = (1.0 / args.ext_dt) as i32;
        // if fps < 50 {
        //     println!("FPS! :{}", fps);
        // }
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let world_ref = &self.world;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            let transform = c.transform;
            for i in 0..WORLD_SIZE as usize {
                if let Some(tile) = &world_ref[i] {
                    let (x, y) = coords(i);
                    let square = rectangle::square(
                        (x * TILE_PIXELS) as f64,
                        (y * TILE_PIXELS) as f64,
                        TILE_PIXELS as f64,
                    );
                    // let color = {
                    //     if tile.velocity.is_zero() {
                    //         [0.0, 1.0, 0.0, 1.0]
                    //     } else {
                    //         [1.0, 0.0, 0.0, 1.0]
                    //     }
                    // };
                    // rectangle(color, square, transform, gl);
                    rectangle(*tile.color(), square, transform, gl);
                }
            }
        });
        let menu_ref = &mut self.element_menu;
        self.gl.draw(args.viewport(), |mut c, gl| {
            // Translate down by the height of the playing area
            c.transform = c.transform.trans(0.0, PLAY_AREA_PIXEL_HEIGHT as f64);
            // Then draw the element selection menu
            menu_ref.draw(c, gl);
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        let mut i = 0;
        while i < UPDATES_PER_FRAME {
            self.world.pause_particles();
            if self.turn % GRAVITY_PERIOD == 0 {
                self.world.apply_gravity();
            }
            if self.turn % REACTION_PERIOD == 0 {
                self.world.apply_periodic_reactions();
            }
            apply_velocity(&mut self.world, &mut self.motion_queue);
            self.turn += 1;
            i += 1;
        }
    }

    pub fn mouse_cursor(&mut self, args: &[f64; 2]) {
        if self.drawing {
            self.selected_pen.draw(&mut self.world, args[0], args[1]);
        } else {
            self.last_mouse_pos = (args[0], args[1]);
        }
    }

    pub fn button(&mut self, args: &ButtonArgs) {
        if let Button::Mouse(MouseButton::Left) = args.button {
            match args.state {
                ButtonState::Press => {
                    if self.last_mouse_pos.1 > PLAY_AREA_PIXEL_HEIGHT as f64 {
                        // Let the menu handle it
                        let (x, y) = (
                            self.last_mouse_pos.0,
                            self.last_mouse_pos.1 - PLAY_AREA_PIXEL_HEIGHT as f64,
                        );
                        if let Some(pen) = self.element_menu.on_click(x, y) {
                            self.selected_pen = pen
                        }
                    } else {
                        self.drawing = true;
                        self.selected_pen.draw(
                            &mut self.world,
                            self.last_mouse_pos.0,
                            self.last_mouse_pos.1,
                        );
                    }
                }
                ButtonState::Release => {
                    self.drawing = false;
                }
            }
        }
    }
}
