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