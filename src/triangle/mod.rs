use bytemuck_derive::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::view::{Draw, DrawArgs, InitArgs};

const SHADER: &str = include_str!("triangle.wgsl");

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
        let color = wgpu::RenderPassColorAttachment {
            view: &args.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                store: wgpu::StoreOp::Store,
            },
        };
        let desc = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(color)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut pass = args.command.begin_render_pass(&desc);
        let pipeline = self.pipeline.as_ref().unwrap();
        let vertex_buffer = pipeline.vertex_buffer.slice(..);
        pass.set_vertex_buffer(0, vertex_buffer);
        pass.set_pipeline(pipeline.pipeline());
        pass.draw(0..pipeline.vertex_count, 0..1);
    }
}

#[derive(Debug)]
struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
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
            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
        };
        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[vertex_buffer],
        };
        let vertices = vec![
            VertexAttributes {
                position: [-1., -1., 0.],
            },
            VertexAttributes {
                position: [0., 1., 0.],
            },
            VertexAttributes {
                position: [1., -1., 0.],
            },
        ];
        let desc = wgpu::util::BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        };
        let vertex_buffer = args.device.create_buffer_init(&desc);
        let swap_chain = args.surface.get_capabilities(args.adapter);
        let swap_chain = swap_chain.formats[0];
        let fragment = wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(swap_chain.into())],
        };
        let desc = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let layout = args.device.create_pipeline_layout(&desc);
        let desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(fragment),
            multiview: None,
            cache: None,
        };
        let pipeline = args.device.create_render_pipeline(&desc);
        Self {
            pipeline,
            vertex_buffer,
            vertex_count: vertices.len() as u32,
        }
    }
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct VertexAttributes {
    pub position: [f32; 3],
}
