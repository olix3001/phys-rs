use phys_rs_render::{PhysApp, WindowSettings, Scene, EguiUI};

fn main() {
    let mut app = PhysApp::new(WindowSettings::new("Phys RS Test".to_string(), (800, 600)));

    let mut scene = Scene::new();
    scene.ui = Some(Box::new(ExampleUI {}));

    app.set_scene(scene);

    app.run();
}

// Simple ui
pub struct ExampleUI {}

impl EguiUI for ExampleUI {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Hello World").show(ctx, |ui| {
            ui.label("Hello World!");
        }); 
    }
}