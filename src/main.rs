use phys_rs_render::{PhysApp, WindowSettings, Scene, ColorPalette, math::Vector2, PhysRenderable, Renderer, Brush, DataCollector, components::{ui::BasicDataUI, physics::draw_spring}};

struct Mass {
    radius: f32,
    position: Vector2,
    direction: Vector2
}

impl PhysRenderable for Mass {
    fn render(&self, brush: &mut Brush, renderer: &mut Renderer, dt: f32, frame: u128) {
        let length = 50.0 + 35.0 * ((frame as f32 / 20.0).sin() + 1.0);
        draw_spring(brush, renderer, self.position, self.position + self.direction * length, 1.0, 120.0, 1.0);
    }

    fn update(&mut self, dt: f32, frame: u128, _data_collector: Option<&mut DataCollector>) {
        // self.position += self.direction * dt;
    }
}

fn main() {
    let mut app = PhysApp::new(WindowSettings::new("Phys RS Test".to_string(), (800, 600)));

    let mut scene = Scene::new();
    scene.add_object(Box::new(Mass {
        radius: 20.0,
        position: Vector2::new(96.0, 96.0),
        direction: Vector2::new(96.0, 10.0).normalize(),
    }));

    scene.ui = Some(Box::new(BasicDataUI::new()));

    app.set_scene(scene);

    app.run();
}