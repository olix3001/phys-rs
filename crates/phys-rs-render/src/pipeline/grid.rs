
use bytemuck::{Pod, Zeroable};
use wgpu::{Buffer, RenderPipeline};

use crate::{vec2::Vector2, renderer::Renderer, color::Color, create_pipeline, write_buffer, render_pass};

use super::PhysPipeline;

const DEFAULT_MAX_GRIDS: usize = 1;
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
        let window_size = (renderer.window.inner_size().width, renderer.window.inner_size().height);
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
        let (instance_buffer, index_buffer, pipeline) = create_pipeline!(grid, Grid { renderer: renderer, max_default: DEFAULT_MAX_GRIDS, index: INDICES });

        Self {
            instances: instance_buffer,
            index_buffer,
            pipeline,

            grids: None,
        }
    }

    fn execute(&mut self, renderer: &mut crate::renderer::Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.grids.is_none() {
            return;
        }

        // set instance buffer
        let grids = self.grids.as_ref().unwrap();
        write_buffer!(grids, self, renderer, encoder);

        // Render pass
        let mut render_pass = render_pass!(encoder, view);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &renderer.globals_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.instances.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw(0..INDICES.len() as u32, 0..grids.len() as u32);

        renderer.draw_calls += 1;
    }
}