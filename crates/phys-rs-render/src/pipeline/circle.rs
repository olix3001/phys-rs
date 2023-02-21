use bytemuck::{Pod, Zeroable};

use crate::{renderer::Renderer, vec2::Vector2, color::Color, create_pipeline, render_pass, write_buffer};

use super::PhysPipeline;

const MAX_DEFAULT_CIRCLES: usize = 100;
const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Circle {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    pub thickness: f32,
}

impl Circle {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32,
        2 => Float32x4,
        3 => Float32
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn create(center: Vector2, radius: f32, color: Color, thickness: f32) -> Self {
        Self {
            center: center.into(),
            radius,
            color: color.into(),
            thickness
        }
    }
}

pub struct CirclePipeline {
    instances: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    pipeline: wgpu::RenderPipeline,

    circles: Vec<Circle>,
}

impl CirclePipeline {
    pub fn set_circles(&mut self, circles: Vec<Circle>) {
        self.circles = circles;
    }

    pub fn add_circle(&mut self, circle: Circle) {
        self.circles.push(circle);
    }

    pub fn clear(&mut self) {
        self.circles.clear();
    }
}

impl PhysPipeline for CirclePipeline {
    fn create(renderer: &mut Renderer) -> Self {
        let (instance_buffer, index_buffer, pipeline) = create_pipeline!(circle, Circle { renderer: renderer, max_default: MAX_DEFAULT_CIRCLES, index: INDICES });

        Self {
            instances: instance_buffer,
            index_buffer,
            pipeline,
            circles: Vec::new(),
        }
    }

    fn execute(&mut self, renderer: &mut Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.circles.is_empty() {
            return;
        }

        let circles = &self.circles;
        write_buffer!(circles, self, renderer, encoder);

        let mut render_pass = render_pass!(encoder, view);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &renderer.globals_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instances.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..circles.len() as u32);
    }
}
