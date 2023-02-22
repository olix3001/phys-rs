use bytemuck::{Pod, Zeroable};

use crate::{renderer::Renderer, vec2::Vector2, color::Color, create_pipeline, render_pass, write_buffer};

use super::PhysPipeline;

const MAX_DEFAULT_QUADS: usize = 100;
const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Quad {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],

    pub thickness: f32,
    pub border_radius: f32,
    pub border_color: [f32; 4],
    pub rotation: f32,
}

impl Quad {
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
        3 => Float32,
        4 => Float32,
        5 => Float32x4,
        6 => Float32
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn create(center: Vector2, size: Vector2, color: Color, thickness: f32, border_radius: f32, border_color: Color, rotation: f32) -> Self {
        Self {
            center: center.into(),
            size: size.into(),
            color: color.into(),
            thickness,
            border_radius,
            border_color: border_color.into(),
            rotation,
        }
    }
}

pub struct QuadPipeline {
    instances: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pipeline: wgpu::RenderPipeline,

    quads: Vec<Quad>,
}

impl QuadPipeline {
    pub fn set_quads(&mut self, quads: Vec<Quad>) {
        self.quads = quads;
    }

    pub fn add_quad(&mut self, quads: Quad) {
        self.quads.push(quads);
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }
}

impl PhysPipeline for QuadPipeline {
    fn create(renderer: &mut Renderer) -> Self {
        let (instance_buffer, index_buffer, pipeline) = create_pipeline!(quad, Quad { renderer: renderer, max_default: MAX_DEFAULT_QUADS, index: INDICES });

        Self {
            instances: instance_buffer,
            index_buffer,
            pipeline,
            quads: Vec::new(),
        }
    }

    fn execute(&mut self, renderer: &mut Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.quads.is_empty() {
            return;
        }

        let quads = &self.quads;
        write_buffer!(quads, self, renderer, encoder);

        let mut render_pass = render_pass!(encoder, view);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &renderer.globals_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instances.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..quads.len() as u32);

        renderer.draw_calls += 1;
    }
}
