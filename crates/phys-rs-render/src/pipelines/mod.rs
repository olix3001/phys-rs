use crate::renderer::Renderer;

mod grid;

pub mod pipelines {
    pub use super::grid::GridPipeline;
}

pub mod elements {
    pub use super::grid::Grid;
}

pub trait PhysPipeline {
    fn execute(&self, renderer: &mut Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);
    fn create(renderer: &mut Renderer) -> Self;
}

// ====< MACROS >====
#[macro_export]
macro_rules! create_pipeline {
    ($name:ident, $id:ident { renderer: $renderer:ident, max_default: $max_default:ident, index: $index:ident }) => {{
        // Create shader
        let shader = $renderer.device.create_shader_module(wgpu::include_wgsl!(concat!("../shaders/", stringify!($name), ".wgsl")));    

        // Create buffers
        let instance_buffer = $renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(concat!(stringify!($id), " instance buffer")),
            size: (std::mem::size_of::<$id>() * $max_default) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create index buffer
        let index_buffer = $renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(concat!(stringify!($id), " index buffer")),
            contents: bytemuck::cast_slice($index),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create render pipeline layout
        let render_pipeline_layout = $renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(concat!(stringify!($id), " pipeline layout")),
            bind_group_layouts: &[&$renderer.globals_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline
        let pipeline = $renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(concat!(stringify!($id), " pipeline")),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Grid::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: $renderer.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        (instance_buffer, index_buffer, pipeline)
    }};
}

#[macro_export]
macro_rules! write_buffer {
    ($inst:ident, $self:ident, $renderer:ident, $encoder:ident) => {{
        let instance_bytes: &[u8] = bytemuck::cast_slice($inst.as_slice());
        let mut instance_buffer = $renderer.staging_belt.write_buffer(
            $encoder,
            &$self.instances,
            0,
            wgpu::BufferSize::new(instance_bytes.len() as u64).unwrap(),
            &$renderer.device);

        instance_buffer.copy_from_slice(instance_bytes);
    }};
}

#[macro_export]
macro_rules! render_pass {
    ($encoder:ident, $view:ident) => {
        $encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(stringify!($self)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &$view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    };
}