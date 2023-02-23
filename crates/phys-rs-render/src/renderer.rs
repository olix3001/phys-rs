use bytemuck::{Zeroable, Pod};
use egui::FontDefinitions;
use egui_wgpu_backend::RenderPass;
use lyon::{geom::Box2D, path::builder::BorderRadii, math::Vector};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{Scene, pipeline::{pipelines::{GridPipeline, CirclePipeline, PolyPipeline, QuadPipeline}, PhysPipeline, elements::{Grid, Circle, Primitive, Quad}}, color::{StandardColorPalette, Color}, vec2::Vector2, PhysApp};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Globals {
    u_resolution: [f32; 2],
}

pub struct Renderer {
    pub window: Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,
    pub platform: egui_winit_platform::Platform,
    pub egui_rpass: RenderPass,

    pub staging_belt: wgpu::util::StagingBelt,

    pub surface_config: wgpu::SurfaceConfiguration,

    pub globals: Globals,
    pub globals_uniform: wgpu::Buffer,
    pub globals_bind_group_layout: wgpu::BindGroupLayout,
    pub globals_bind_group: wgpu::BindGroup,

    pub brush: Option<Brush>,

    has_to_update_globals: bool,

    pub encoder: Option<wgpu::CommandEncoder>,
    pub view: Option<wgpu::TextureView>,
    pub frame: Option<wgpu::SurfaceTexture>,

    pub draw_calls: u32,
    pub ldt: f32, // Last delta time

    pub avg_update_time: f32,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )).unwrap();

        let surface_format = surface.get_supported_formats(&adapter)[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &surface_config);

        // Egui
        let platform = egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let egui_rpass = RenderPass::new(&device, surface_format, 1);

        let staging_belt = wgpu::util::StagingBelt::new(10 * 1024);

        // Globals
        let globals = Globals {
            u_resolution: [window.inner_size().width as f32, window.inner_size().height as f32],
        };

        // Create globals uniform buffer
        let globals_uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals Uniform Buffer"),
            contents: bytemuck::cast_slice(&[globals]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create globals bind group layout
        let globals_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create globals bind group
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globals Bind Group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_uniform.as_entire_binding(),
            }],
        });


        let mut s = Self {
            window,
            surface,
            device,
            queue,
            platform,
            format: surface_format,
            egui_rpass,
            staging_belt,

            globals,
            globals_uniform,
            globals_bind_group_layout,
            globals_bind_group,

            surface_config,

            brush: None,
            has_to_update_globals: false,

            encoder: None,
            view: None,
            frame: None,

            draw_calls: 0,
            ldt: 0.0,
            avg_update_time: 0.0,
        };

        // Brush
        let brush = Brush::new(&mut s);

        Self {
            brush: Some(brush),
            ..s
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
        
        self.globals.u_resolution = [new_size.width as f32, new_size.height as f32];
        self.has_to_update_globals = true;
    }

    pub fn update_uniforms(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let binding = [self.globals];
        let global_bytes = bytemuck::cast_slice(&binding);
        let mut globals_buffer = self.staging_belt.write_buffer(
            encoder,
            &self.globals_uniform,
            0,
            wgpu::BufferSize::new(global_bytes.len() as u64).unwrap(),
            &self.device);
        globals_buffer.copy_from_slice(global_bytes);
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn render_begin(&mut self, scene: &mut Scene, start_time: std::time::Instant) {
        // clear the screen
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        if self.has_to_update_globals {
            self.update_uniforms(&mut encoder);
            self.has_to_update_globals = false;
        }

        {
            let mut _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(scene.background_color.into()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        // Draw pipelines
        if self.brush.is_some() {
            let mut pipelines = self.brush.take().unwrap();
            pipelines.execute_once(self, &mut encoder, &view);
            self.brush = Some(pipelines);
        }

        // draw egui
        self.platform.update_time(start_time.elapsed().as_secs_f64());
        self.platform.begin_frame();

        self.encoder = Some(encoder);
        self.view = Some(view);
        self.frame = Some(frame);
    }

    pub fn execute_brush(&mut self, brush: &mut Brush) {
        let view = self.view.take().unwrap();
        let mut encoder = self.encoder.take().unwrap();

        brush.execute(self, &mut encoder, &view);
        brush.clear();

        self.encoder = Some(encoder);
        self.view = Some(view);
    }

    pub fn render_end(&mut self, scene: &mut Scene) {
        if let(Some(mut encoder), Some(view), Some(frame)) = (self.encoder.take(), self.view.take(), self.frame.take()) {
            // Clear pipelines
            if self.brush.is_some() {
                let mut pipelines = self.brush.take().unwrap();
                pipelines.execute(self, &mut encoder, &view);
                pipelines.clear();
                self.brush = Some(pipelines)
            }

            // draw UI
            if scene.ui.is_some() {
                scene.ui.as_mut().unwrap().ui(&self.platform.context(), self);
            }
            // Finish drawing egui
            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: self.surface_config.width,
                physical_height: self.surface_config.height,
                scale_factor: self.window.scale_factor() as f32,
            };
            let full_output = self.platform.end_frame(Some(&self.window));
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);
            let tdelta: egui::TexturesDelta = full_output.textures_delta;
            self.egui_rpass.add_textures(&self.device, &self.queue, &tdelta).expect("add texture ok");
            self.egui_rpass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

            // execute egui render pass
            self.egui_rpass.execute(
                &mut encoder,
                &view,
                &paint_jobs,
                &screen_descriptor,
                None
            ).unwrap();

            // Clear the egui textures
            self.egui_rpass.remove_textures(tdelta).expect("remove textures ok");


            // Finish drawing
            self.staging_belt.finish();
            self.queue.submit(std::iter::once(encoder.finish()));
            frame.present();

            self.staging_belt.recall();
            self.draw_calls = 0;
        }
    }


    // ====< DATA >====
    pub fn get_window_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }
}


// ====< BRUSH >====
pub struct Brush {
    // pipelines
    pub grid_pipeline: GridPipeline,
    pub circle_pipeline: CirclePipeline,
    pub polygon_pipeline: PolyPipeline,
    pub quad_pipeline: QuadPipeline,
}

impl Brush {
    pub fn new(renderer: &mut Renderer) -> Self {
        let mut grid = GridPipeline::create(renderer);
        grid.set_grids(vec![
            Grid::fullscreen(StandardColorPalette::GRID, 30.0, 1.0, 5)
        ]);
        Self {
            grid_pipeline: grid,
            circle_pipeline: CirclePipeline::create(renderer),
            polygon_pipeline: PolyPipeline::create(renderer),
            quad_pipeline: QuadPipeline::create(renderer),
        }
    }

    pub fn execute_once(&mut self, renderer: &mut Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.grid_pipeline.execute(renderer, encoder, view);
    }
    pub fn execute(&mut self, renderer: &mut Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.circle_pipeline.execute(renderer, encoder, view);
        self.polygon_pipeline.execute(renderer, encoder, view);
        self.quad_pipeline.execute(renderer, encoder, view);
    }

    // ====< BASIC >====
    pub fn clear(&mut self) {
        self.circle_pipeline.clear();
        self.polygon_pipeline.clear();
        self.quad_pipeline.clear();
    }

    // ====< PRIMITIVES >====
    // Circle
    pub fn draw_circle(&mut self, center: Vector2, radius: f32, color: Color, thickness: f32) {
        self.circle_pipeline.add_circle(Circle::create(center, radius, color, thickness));
    }
    pub fn draw_circle_filled(&mut self, center: Vector2, radius: f32, color: Color) {
        self.circle_pipeline.add_circle(Circle::create(center, radius, color, 0.0));
    }

    // ====< POLYGON >====
    pub fn draw_quad_filled(&mut self, a: Vector2, b: Vector2, color: Color) {
        self.polygon_pipeline.tesselate_fn(|builder| {
            builder.add_rectangle(
                &Box2D { min: a.into(), max: b.into() },
                lyon::path::Winding::Positive
            );
        }, Some(Primitive {
            color: color.into(),
            angle: 0.0,
            origin: Vector2::zero().into(),
            ..Default::default()
        }))
    }

    pub fn draw_rquad_filled(&mut self, a: Vector2, b: Vector2, color: Color, radius: f32) {
        let center = (a + b) / 2.0;
        let size = b - a;
        self.quad_pipeline.add_quad(Quad::create(center, size, color, 0.0, radius, StandardColorPalette::TRANSPARENT, 0.5));
    }

    // Raw quad
    pub fn _draw_quad_border_raw(&mut self, center: Vector2, size: Vector2, color: Color, border_thickness: f32, border_color: Color, angle: f32, radius: f32) {
        self.quad_pipeline.add_quad(Quad::create(center, size, color, border_thickness, radius, border_color, angle));
    }


    // ====< LINES >====
    pub fn draw_line(&mut self, a: Vector2, b: Vector2, thickness: f32, color: Color) {
        let center = (a + b) / 2.0;
        let length = (b - a).length();
        let angle = -(b - a).angle();
        let size = Vector2::new(length, thickness);

        self.quad_pipeline.add_quad(Quad::create(center, size, color, 0.0, 0.0, StandardColorPalette::TRANSPARENT, angle));
    }

    pub fn draw_line_rounded(&mut self, a: Vector2, b: Vector2, thickness: f32, color: Color) {
        let center = (a + b) / 2.0;
        let length = (b - a).length();
        let angle = -(b - a).angle();
        let size = Vector2::new(length, thickness);

        self.quad_pipeline.add_quad(Quad::create(center, size, color, 0.0, size.y / 2.0, StandardColorPalette::TRANSPARENT, angle));
    }

    // ====< FLUSH >====
    pub fn flush(&mut self, renderer: &mut Renderer) {
        renderer.execute_brush(self); 
        self.clear();
    }
}