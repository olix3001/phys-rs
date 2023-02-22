/*
    BASIC DATA UI
    - FPS
    - Window size
    - Mouse position
    - Delta time
    - Frame count
    - Draw calls count
*/

use crate::{Renderer, EguiUI, PhysApp};

pub struct BasicDataUI { }

impl BasicDataUI {
    pub fn new() -> Self {
        Self { }
    }
}

impl EguiUI for BasicDataUI {
    fn ui(&mut self, ctx: &egui::Context, renderer: &Renderer) {
        // window
        egui::Window::new("Debug data").show(ctx, |ui| {
            // fps
            ui.label(format!("FPS: {}", 1.0/renderer.ldt));

            // delta time
            ui.label(format!("Delta time: {}", renderer.ldt));

            // window size
            let window_size = renderer.get_window_size();
            ui.label(format!("Window size: {}x{}", window_size.0, window_size.1));

            // TODO: mouse position
            // ui.label(format!("Mouse position: {}x{}", self.renderer.mouse_position.0, self.renderer.mouse_position.1));

            // draw calls count
            ui.label(format!("Draw calls: {}", renderer.draw_calls + 1));
        });
    }
}
