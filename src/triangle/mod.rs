use std::{
    f32::consts::PI,
    time::{SystemTime, UNIX_EPOCH},
};

use bytemuck_derive::{Pod, Zeroable};
use num_traits::Float;
use wgpu::util::DeviceExt;

use crate::view::{Draw, DrawArgs, InitArgs};

const SHADER: &str = include_str!("triangle.wgsl");
const IS_WIREFRAME: bool = false;
const PER_PERIOD: usize = 2 << 10;

#[derive(Debug)]
pub struct DrawTriangle {
    pipeline: Option<Pipeline>,
}
impl DrawTriangle {
    pub fn new() -> Self {
        Self { pipeline: None }
    }
}
impl Default for DrawTriangle {
    fn default() -> Self {
        Self::new()
    }
}
impl Draw for DrawTriangle {
    fn init(&mut self, args: InitArgs<'_>) {
        self.pipeline = Some(Pipeline::new(args));
    }

    fn draw(&mut self, args: DrawArgs<'_>) {
        let pipeline = self.pipeline.as_ref().unwrap();
        let gray = wgpu::Color {
            r: 0.2,
            g: 0.3,
            b: 0.3,
            a: 1.0,
        };
        let background = wgpu::RenderPassColorAttachment {
            view: &args.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(gray),
                store: wgpu::StoreOp::Store,
            },
        };
        let green = normalize_neg_pos_1(f32::sin(
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % (PER_PERIOD as u128)) as f32
                * 2.
                * PI
                / PER_PERIOD as f32,
        ));
        dbg!(green);
        let uniform = Uniform { green };
        args.queue
            .write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
        let desc = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(background)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut pass = args.command.begin_render_pass(&desc);
        let vertex_buffer = pipeline.vertex_buffer.slice(..);
        pass.set_vertex_buffer(0, vertex_buffer);
        let index_buffer = pipeline.index_buffer.slice(..);
        pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
        pass.set_pipeline(pipeline.pipeline());
        pass.set_bind_group(0, &pipeline.bind_group, &[]);
        pass.draw_indexed(0..pipeline.index_count, 0, 0..1);
    }
}

#[derive(Debug)]
struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
impl Pipeline {
    pub fn new(args: InitArgs<'_>) -> Self {
        let shader = wgpu::ShaderSource::Wgsl(SHADER.into());
        let desc = wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader,
        };
        let shader = args.device.create_shader_module(desc);
        let vertex_buffer = wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<VertexAttributes>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
        };
        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[vertex_buffer],
        };
        let mesh = triangle();
        // let mesh = rectangle();
        let desc = wgpu::util::BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let vertex_buffer = args.device.create_buffer_init(&desc);
        let desc = wgpu::util::BufferInitDescriptor {
            label: Some("indices"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        };
        let index_buffer = args.device.create_buffer_init(&desc);
        let swap_chain = args.surface.get_capabilities(args.adapter);
        let swap_chain = swap_chain.formats[0];
        let fragment = wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(swap_chain.into())],
        };
        let desc = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };
        let bind_group = args.device.create_bind_group_layout(&desc);
        let desc = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group],
            push_constant_ranges: &[],
        };
        let layout = args.device.create_pipeline_layout(&desc);
        let polygon_mode = if IS_WIREFRAME {
            wgpu::PolygonMode::Line
        } else {
            wgpu::PolygonMode::default()
        };
        let desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex,
            primitive: wgpu::PrimitiveState {
                polygon_mode,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(fragment),
            multiview: None,
            cache: None,
        };
        let pipeline = args.device.create_render_pipeline(&desc);
        let layout = pipeline.get_bind_group_layout(0);
        let desc = wgpu::BufferDescriptor {
            label: Some("uniform"),
            size: core::mem::size_of::<Uniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let uniform_buffer = args.device.create_buffer(&desc);
        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        };
        let bind_group = args.device.create_bind_group(&desc);
        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            uniform_buffer,
            bind_group,
        }
    }
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniform {
    pub green: f32,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct VertexAttributes {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

struct Mesh {
    pub vertices: Vec<VertexAttributes>,
    pub indices: Vec<u32>,
}

#[allow(unused)]
fn triangle() -> Mesh {
    let vertices = vec![
        VertexAttributes {
            position: [-1., -1., 0.],
            color: [1., 0., 0.],
        },
        VertexAttributes {
            position: [0., 1., 0.],
            color: [0., 1., 0.],
        },
        VertexAttributes {
            position: [1., -1., 0.],
            color: [0., 0., 1.],
        },
    ];
    let indices = vec![0, 1, 2];
    Mesh { vertices, indices }
}

#[allow(unused)]
fn rectangle() -> Mesh {
    let color = [1.0, 0.5, 0.2];
    let vertices = vec![
        VertexAttributes {
            position: [0.5, 0.5, 0.],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, 0.],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, 0.],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, 0.],
            color,
        },
    ];
    let vertex_pos = QuadVertexPos {
        top_right: 0,
        bottom_right: 1,
        top_left: 2,
        bottom_left: 3,
    };
    let indices = quad_indices(vertex_pos).into();
    Mesh { vertices, indices }
}

#[derive(Debug, Clone, Copy)]
struct QuadVertexPos {
    pub top_right: u32,
    pub bottom_right: u32,
    pub top_left: u32,
    pub bottom_left: u32,
}
fn quad_indices(vertex_pos: QuadVertexPos) -> [u32; 6] {
    [
        vertex_pos.top_right,
        vertex_pos.bottom_right,
        vertex_pos.top_left, //
        vertex_pos.bottom_right,
        vertex_pos.bottom_left,
        vertex_pos.top_left,
    ]
}

fn normalize_neg_pos_1<T: Float>(v: T) -> T {
    let one = T::from(1.).unwrap();
    let two = T::from(2.).unwrap();
    (v + one) / two
}
