use color::{Color, StandardColorPalette};
use winit::{window::{WindowBuilder}, dpi::PhysicalSize, event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent}};

mod renderer;
mod color;
mod vec2;
mod pipeline;

use renderer::Renderer;

// ====< PHYS APP >====
pub struct PhysApp {
    pub window_settings: WindowSettings,
    pub event_loop: EventLoop<()>,
    pub renderer: Renderer,
    pub scene: Scene,
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
        let scene = Scene::new();

        Self {
            window_settings,
            event_loop,
            renderer,
            scene,
        }
    }

    pub fn set_scene(&mut self, scene: Scene) {
        self.scene = scene;
    }

    pub fn run(mut self) -> ! {
        let start_time = std::time::Instant::now();
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
                    self.renderer.render(&mut self.scene, start_time);
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
pub struct Scene {
    pub objects: Vec<Box<dyn PhysRenderable>>,
    pub ui: Option<Box<dyn EguiUI>>,
    pub background_color: Color
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            ui: None,
            background_color: StandardColorPalette::BACKGROUND
        }
    }
}
impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_object(&mut self, object: Box<dyn PhysRenderable>) {
        self.objects.push(object);
    }
}

// ====< TRAITS >====
pub trait PhysRenderable {
    fn render(&self, renderer: &mut Renderer);
}

pub trait EguiUI {
    fn ui(&mut self, ctx: &egui::Context);
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