use bytemuck::{Zeroable, Pod};
use lyon::{path::{Path, path::Builder}, lyon_tessellation::{FillOptions, VertexBuffers, BuffersBuilder}};

use crate::{write_buffer, render_pass};

use super::PhysPipeline;

const MAX_DEFAULT_VERTICES: usize = 1000;
const MAX_DEFAULT_PRIMITIVES: usize = 2;


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    prim_id: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Primitive {
    pub color: [f32; 4],
    pub angle: f32,
    pub origin: [f32; 2],

    pub indices_count: u32,
}

impl Default for Primitive {
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0, 0.0],
            angle: 0.0,
            origin: [0.0, 0.0],
            indices_count: 0,
        }
    }
}

impl Primitive {
    pub fn create(color: [f32; 4], angle: f32, origin: [f32; 2]) -> Self {
        Self {
            color,
            angle,
            origin,
            ..Default::default()
        }
    }

    pub fn no_rotation(color: [f32; 4], origin: [f32; 2]) -> Self {
        Self {
            color,
            origin,
            ..Default::default()
        }
    }
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Uint32
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct PolyPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vbo: wgpu::Buffer,
    pub ibo: wgpu::Buffer,
    pub index_count: u32,

    // lyon
    pub fill_tess: lyon::tessellation::FillTessellator,
    pub stroke_tess: lyon::tessellation::StrokeTessellator,

    pub geometry: VertexBuffers<Vertex, u16>,

    // uniforms
    pub primitives_buffer: wgpu::Buffer,
    pub primitives_bind_group: wgpu::BindGroup,
    pub primitives_bind_group_layout: wgpu::BindGroupLayout,

    // primitives
    pub primitives: Vec<Primitive>,
    i_index: usize,
    v_count: u32,
}

impl PolyPipeline {
    pub fn tesselate_fn(&mut self, builder: impl FnOnce(&mut Builder), primitive: Option<Primitive>) {
        let mut builder_l = Path::builder();
        builder(&mut builder_l);
        self.tesselate(&builder_l.build(), primitive);
    }

    pub fn tesselate(&mut self, path: &Path, primitive: Option<Primitive>) {

        let size_before = self.geometry.indices.len();
        self.fill_tess.tessellate_path(
            path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut self.geometry, |vertex: lyon::tessellation::FillVertex| {
                Vertex {
                    position: vertex.position().to_array(),
                    prim_id: self.i_index as u32,
                }
            })
        ).unwrap();

        self.v_count += (self.geometry.indices.len() - size_before) as u32;

        if let Some(mut primitive) = primitive {
            primitive.indices_count = self.v_count;
            self.primitives.push(primitive); 
            self.v_count = 0;
            self.i_index += 1;
        }
    }

    pub fn clear(&mut self) {
        self.geometry.indices.clear();
        self.geometry.vertices.clear();
        self.primitives.clear();

        self.v_count = 0;
        self.i_index = 0;
    }
}

impl PhysPipeline for PolyPipeline {
    fn execute(&mut self, renderer: &mut crate::Renderer, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.geometry.vertices.is_empty() {
            return;
        }

        // Copy buffers
        write_buffer!(self.geometry.vertices, self, renderer, encoder, self.vbo);
        write_buffer!(self.geometry.indices, self, renderer, encoder, self.ibo);
        write_buffer!(self.primitives, self, renderer, encoder, self.primitives_buffer);
        // let mut primitive_buffer = renderer.staging_belt.write_buffer(
        //     encoder,
        //     &self.primitives_buffer,
        //     0,
        //     wgpu::BufferSize::new(self.primitives_buffer.size()).unwrap(),
        //     &renderer.device);

        // primitive_buffer.copy_from_slice(primitive_bytes);


        // Draw
        {
            let mut index_offset = 0;
            let mut render_pass = render_pass!(encoder, view);


            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &renderer.globals_bind_group, &[]);
            render_pass.set_bind_group(1, &self.primitives_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vbo.slice(..));
            render_pass.set_index_buffer(self.ibo.slice(..), wgpu::IndexFormat::Uint16);


            let mut i = 0;
            for primitive in self.primitives.iter() {
                let offset = i;

                render_pass.draw_indexed(index_offset..index_offset+primitive.indices_count as u32, 0, 0..1);

                index_offset += primitive.indices_count;
                i += 1;
            }
        }

        self.clear();
        // ::std::process::exit(0);
    }

    fn create(renderer: &mut crate::Renderer) -> Self {
        // Create tessellators
        let fill_tess = lyon::tessellation::FillTessellator::new();
        let stroke_tess = lyon::tessellation::StrokeTessellator::new();

        // Create builder

        // Create shader
        let shader = renderer.device.create_shader_module(wgpu::include_wgsl!("../shaders/poly.wgsl"));

        // Create empty buffers
        let vbo = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Poly vertex buffer"),
            size: (MAX_DEFAULT_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ibo = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Poly index buffer"),
            size: (MAX_DEFAULT_VERTICES * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create primitives uniform buffer
        let primitives_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Poly primitives buffer"),
            size: (MAX_DEFAULT_PRIMITIVES * std::mem::size_of::<Primitive>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create primitives bind group layout
        let primitives_bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Poly primitives bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new((MAX_DEFAULT_PRIMITIVES * std::mem::size_of::<Primitive>()) as u64),
                },
                count: None,
            }],
        });

        // Create primitives bind group
        let primitives_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Poly primitives bind group"),
            layout: &primitives_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: primitives_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline layout
        let render_pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Poly pipeline layout"),
            bind_group_layouts: &[&renderer.globals_bind_group_layout, &primitives_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline
        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(concat!(stringify!($id), " pipeline")),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
            pipeline,
            vbo,
            ibo,
            index_count: 0,
            fill_tess,
            stroke_tess,
            primitives_buffer,
            primitives_bind_group,
            primitives_bind_group_layout,

            geometry: VertexBuffers::new(),
            primitives: Vec::with_capacity(MAX_DEFAULT_PRIMITIVES),

            v_count: 0,
            i_index: 0
        }
    }
}