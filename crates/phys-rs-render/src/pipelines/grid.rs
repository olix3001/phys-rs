use std::ops::DerefMut;

use bytemuck::{Pod, Zeroable};
use wgpu::{Buffer, RenderPipeline, util::DeviceExt};

use crate::{vec2::Vector2, renderer::Renderer, color::Color};

use super::PhysPipeline;

const DEFAULT_MAX_GRIDS: usize = 10;
const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Grid {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
    pub color: [f32; 4],

    pub spacing: f32,
    pub thickness: f32,
    pub subdivisions: u32,
}

impl Grid {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
        3 => Float32,
        4 => Float32,
        5 => Uint32
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn create(renderer: &Renderer, pos: Vector2, size: Vector2, color: Color, spacing: f32, thickness: f32, subdivisions: u32) -> Self {
        let window_size = (renderer.window.inner_size().width as u32, renderer.window.inner_size().height as u32);
        Self {
            top_left: pos.to_ndc(window_size).into(),
            bottom_right: (pos + size).to_ndc(window_size).into(),
            color: color.into(),
            spacing,
            thickness,
            subdivisions,
        }
    }

    pub fn fullscreen(color: Color, spacing: f32, thickness: f32, subdivisions: u32) -> Self {
        Self {
            top_left: [-1.0, 1.0],
            bottom_right: [1.0, -1.0],
            color: color.into(),
            spacing,
            thickness,
            subdivisions,
        }
    }
}

pub struct GridPipeline {
    instances: Buffer,
    index_buffer: Buffer,
    
    pipeline: RenderPipeline,

    grids: Option<Vec<Grid>>,
}

impl GridPipeline {
    pub fn set_grids(&mut self, grids: Vec<Grid>) {
        self.grids = Some(grids);
    }
}

impl PhysPipeline for GridPipeline {
    fn create(renderer: &mut crate::renderer::Renderer) -> Self {
        // Create shader
        let shader = renderer.device.create_shader_module(wgpu::include_wgsl!("../shaders/grid.wgsl"));

        // Create buffers
        let instance_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid instance buffer"),
            size: (std::mem::size_of::<crate::pipelines::grid::Grid>() * crate::pipelines::grid::DEFAULT_MAX_GRIDS) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create index buffer
        let index_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid index buffer"),
            contents: bytemuck::cast_slice(crate::pipelines::grid::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create render pipeline layout
        let render_pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid pipeline layout"),
            bind_group_layouts: &[&renderer.globals_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline
        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid pipeline"),
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
                    format: renderer.format,
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

        Self {
            instances: instance_buffer,
            index_buffer,
            pipeline,

            grids: None,
        }
    }

    fn execute(&self, renderer: &mut crate::renderer::Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.grids.is_none() {
            return;
        }

        // set instance buffer
        let grids = self.grids.as_ref().unwrap();
        let instance_bytes: &[u8] = bytemuck::cast_slice(grids.as_slice());
        let mut instance_buffer = renderer.staging_belt.write_buffer(
            encoder,
            &self.instances,
            0,
            wgpu::BufferSize::new(instance_bytes.len() as u64).unwrap(),
            &renderer.device);

        instance_buffer.copy_from_slice(instance_bytes);

        // Render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Grid render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &renderer.globals_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instances.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw(0..INDICES.len() as u32, 0..grids.len() as u32);

    }
}