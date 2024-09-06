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
        pass.set_pipeline(self.pipeline.as_ref().unwrap().pipeline());
        pass.draw(0..3, 0..1);
    }
}

#[derive(Debug)]
struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}
impl Pipeline {
    pub fn new(args: InitArgs<'_>) -> Self {
        let shader = wgpu::ShaderSource::Wgsl(SHADER.into());
        let desc = wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader,
        };
        let shader = args.device.create_shader_module(desc);
        let desc = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        };
        let layout = args.device.create_pipeline_layout(&desc);
        let vertex = wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        };
        let swap_chain = args.surface.get_capabilities(args.adapter);
        let swap_chain = swap_chain.formats[0];
        let fragment = wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(swap_chain.into())],
        };
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
        Self { pipeline }
    }
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
