use std::thread;

use color::{Color, StandardColorPalette};
use winit::{window::{WindowBuilder}, dpi::PhysicalSize, event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent}, platform::run_return::EventLoopExtRunReturn};

mod renderer;
mod color;
mod vec2;
mod pipeline;

pub mod components;

pub use renderer::{Renderer, Brush};

// ====< EXPORTS >====
pub use color::StandardColorPalette as ColorPalette;
pub mod math {
    pub use crate::vec2::Vector2;
}

// ====< PHYS APP >====
pub struct PhysApp {
    pub window_settings: WindowSettings,
    pub event_loop: EventLoop<()>,
    pub renderer: Renderer,
    pub scene: Scene,
    pub updates_per_frame: u32,
    pub avg_update_time: f32,
}

impl PhysApp {
    pub fn new(window_settings: WindowSettings) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(window_settings.title.clone())
            .with_inner_size(PhysicalSize::new(window_settings.size.0, window_settings.size.1))
            .build(&event_loop)
            .unwrap();
        let renderer = Renderer::new(window);
        let scene = Scene::default();

        Self {
            window_settings,
            event_loop,
            renderer,
            scene,
            updates_per_frame: 1,
            avg_update_time: 0.0,
        }
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.scene = scene;
    }

    pub fn run(mut self) -> ! {
        let start_time = std::time::Instant::now();
        let mut last_frame = std::time::Instant::now();
        let mut frame: u128 = 0;
        self.event_loop.run(move |event, _, control_flow| {
            self.renderer.handle_event(&event);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(physical_size);
                    }
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    self.renderer.render_begin(&mut self.scene, start_time);
                    // draw
                    let dt = last_frame.elapsed().as_secs_f32();
                    self.renderer.ldt = dt;

                    let mut brush = self.renderer.brush.take().unwrap();
                    for object in self.scene.objects.iter_mut() {
                        object.render(&mut brush, &mut self.renderer, dt, frame);
                    }
                    self.renderer.brush = Some(brush);

                    // update
                    let dt = dt / self.updates_per_frame as f32;
                    let update_start = std::time::Instant::now();
                    for _ in 0..self.updates_per_frame {
                        for object in self.scene.objects.iter_mut() {
                            object.update(dt, frame, None);
                        }
                    }
                    self.renderer.avg_update_time = update_start.elapsed().as_secs_f32() / self.updates_per_frame as f32;

                    self.renderer.render_end(&mut self.scene);
                    last_frame = std::time::Instant::now();
                    frame += 1;
                }
                Event::MainEventsCleared => {
                    self.renderer.window.request_redraw();
                }
                _ => (),
            }
        })
    }
}

// ====< WINDOW SETTINGS >====
pub struct WindowSettings {
    pub title: String,
    pub size: (u32, u32),
}

impl WindowSettings {
    pub fn new(title: String, size: (u32, u32)) -> Self {
        Self { title, size }
    }
}

// ====< SCENE >====
// Scene, Brush, dt, frame count
type UpdateFn = Box<dyn FnMut(&mut Scene, &mut Brush, f32, u128)>;
pub struct Scene {
    pub ui: Option<Box<dyn EguiUI>>,
    pub background_color: Color,

    pub objects: Vec<Box<dyn PhysRenderable>>,

    // pub update: Option<UpdateFn>,

}
impl Default for Scene {
    fn default() -> Self {
        Self {
            ui: None,
            background_color: StandardColorPalette::BACKGROUND,
            objects: Vec::new(),
            // update: None,
        }
    }
}
impl Scene {
    pub fn new() -> Self {
        Self {
            // update,
            ..Default::default()
        }
    }

    pub fn add_object(&mut self, object: Box<dyn PhysRenderable>) {
        self.objects.push(object);
    }
}

// ====< DATA COLLECTOR >====
pub struct DataCollector {
}

// ====< TRAITS >====
pub trait PhysRenderable {
    fn render(&self, brush: &mut Brush, renderer: &mut Renderer, dt: f32, frame: u128);
    fn update(&mut self, dt: f32, frame: u128, data_collector: Option<&mut DataCollector>);
}

pub trait EguiUI {
    fn ui(&mut self, ctx: &egui::Context, renderer: &Renderer);
}

// ====< UTILS >====
pub fn ndc_to_screen_space(ndc: (f32, f32), window_size: (u32, u32)) -> (f32, f32) {
    let x = (ndc.0 + 1.0) * 0.5 * window_size.0 as f32;
    let y = (ndc.1 + 1.0) * 0.5 * window_size.1 as f32;
    (x, y)
}

pub fn to_ndc_vsize(px: f32, window_size: (u32, u32)) -> f32 {
    px / window_size.0 as f32 * 2.0
}
pub fn to_ndc_hsize(px: f32, window_size: (u32, u32)) -> f32 {
    px / window_size.1 as f32 * 2.0
}