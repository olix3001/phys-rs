use phys_rs_render::{PhysApp, WindowSettings, Scene, EguiUI, ColorPalette, math::Vector2, PhysRenderable, Renderer, Brush, DataCollector};

struct Mass {
    radius: f32,
    position: Vector2,
    direction: Vector2,
}

impl PhysRenderable for Mass {
    fn render(&self, brush: &mut Brush, dt: f32, frame: u128) {
        brush.draw_circle_filled(self.position, self.radius, ColorPalette::RED);
    }

    fn update(&mut self, dt: f32, frame: u128, _data_collector: Option<&mut DataCollector>) {
        self.position += self.direction * dt;
        self.direction += Vector2::new(0.0, 0.5);
    }
}

fn main() {
    let mut app = PhysApp::new(WindowSettings::new("Phys RS Test".to_string(), (800, 600)));

    let mut scene = Scene::new();
    scene.add_object(Box::new(Mass {
        radius: 20.0,
        position: Vector2::new(100.0, 100.0),
        direction: Vector2::new(100.0, 0.0),
    }));

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