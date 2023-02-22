use phys_rs_render::{PhysApp, WindowSettings, Scene, ColorPalette, math::Vector2, PhysRenderable, Renderer, Brush, DataCollector, components::ui::BasicDataUI};

struct Mass {
    radius: f32,
    position: Vector2,
    direction: Vector2,
}

impl PhysRenderable for Mass {
    fn render(&self, brush: &mut Brush, renderer: &mut Renderer, dt: f32, frame: u128) {
        // brush.draw_circle_filled(self.position, self.radius, ColorPalette::RED);
        brush.draw_circle_filled(self.position + Vector2::new(50.0, 10.0), self.radius, ColorPalette::BLUE);
        brush.draw_aarquad_filled(self.position, self.position + Vector2::new(100.0, 100.0), ColorPalette::RED, 15.0);
    }

    fn update(&mut self, dt: f32, frame: u128, _data_collector: Option<&mut DataCollector>) {
        // self.position += self.direction * dt;
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

    scene.ui = Some(Box::new(BasicDataUI::new()));

    app.set_scene(scene);

    app.run();
}