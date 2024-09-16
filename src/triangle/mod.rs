use std::{
    f64::consts::PI,
    time::{SystemTime, UNIX_EPOCH},
};

use bytemuck_derive::{Pod, Zeroable};
use num_traits::Float;
use wgpu::util::DeviceExt;

use crate::{
    transform::{perspective, rotate, translate},
    view::{Draw, DrawArgs, InitArgs},
};

const SHADER: &str = include_str!("triangle.wgsl");
const WALL: &[u8] = include_bytes!("wall.jpg");
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
        pipeline.draw(args);
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
        let texture = Texture::new(args.device, WALL);
        texture.register(args.queue);
        let shader = wgpu::ShaderSource::Wgsl(SHADER.into());
        let desc = wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader,
        };
        let shader = args.device.create_shader_module(desc);
        let vertex_buffer = wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<VertexAttributes>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &VertexAttributes::layout(),
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
        let desc = wgpu::BufferDescriptor {
            label: Some("uniform"),
            size: core::mem::size_of::<Uniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let uniform_buffer = args.device.create_buffer(&desc);
        let bind_group_bindings = [
            (
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ),
            (
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: texture.texture_layout(),
                    count: None,
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                },
            ),
            (
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: texture.sampler_layout(),
                    count: None,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(texture.sampler()),
                },
            ),
        ];
        let desc = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &bind_group_bindings
                .iter()
                .map(|(x, _)| *x)
                .collect::<Vec<_>>(),
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
        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &bind_group_bindings
                .iter()
                .map(|(_, x)| x.clone())
                .collect::<Vec<_>>(),
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

    pub fn draw(&self, args: DrawArgs<'_>) {
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
        let sin = normalize_neg_pos_1(f64::sin(
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % (PER_PERIOD as u128)) as f64
                * 2.
                * PI
                / PER_PERIOD as f64,
        ));
        dbg!(sin);
        // let trans = {
        //     let translate = translate([0.5, -0.5, 0.0]);
        //     let angle = sin * PI * 2.;
        //     let rotate = rotate([0.0, 0.0, 1.0], angle);
        //     matrix_mul(&rotate, &translate)
        // };
        // dbg!(&trans);
        let model = rotate([1., 0., 0.], PI / 3.);
        let view = translate([0., 0., -3.]);
        let projection = perspective(PI / 4., 800. / 600., 0.1, 100.);
        let uniform = Uniform {
            model: model.transpose().into_buffer().map(|x| x as f32),
            view: view.transpose().into_buffer().map(|x| x as f32),
            projection: projection.transpose().into_buffer().map(|x| x as f32),
            _padding: [0; 3],
            sin: sin as f32,
        };
        args.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
        let desc = wgpu::CommandEncoderDescriptor { label: None };
        let mut command = args.device.create_command_encoder(&desc);
        {
            let desc = wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(background)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut pass = command.begin_render_pass(&desc);
            let vertex_buffer = self.vertex_buffer.slice(..);
            pass.set_vertex_buffer(0, vertex_buffer);
            let index_buffer = self.index_buffer.slice(..);
            pass.set_index_buffer(index_buffer, wgpu::IndexFormat::Uint32);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw_indexed(0..self.index_count, 0, 0..1);
        }
        args.queue.submit([command.finish()]);
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniform {
    pub model: [f32; 16],
    pub view: [f32; 16],
    pub projection: [f32; 16],
    pub _padding: [u32; 3],
    pub sin: f32,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct VertexAttributes {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}
impl VertexAttributes {
    pub fn layout() -> Vec<wgpu::VertexAttribute> {
        wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2,
        ]
        .into()
    }
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
            tex_coord: [0., 0.],
        },
        VertexAttributes {
            position: [0., 1., 0.],
            color: [0., 1., 0.],
            tex_coord: [0.5, 1.],
        },
        VertexAttributes {
            position: [1., -1., 0.],
            color: [0., 0., 1.],
            tex_coord: [1., 0.],
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
            tex_coord: [1., 1.],
        },
        VertexAttributes {
            position: [0.5, -0.5, 0.],
            color,
            tex_coord: [1., 0.],
        },
        VertexAttributes {
            position: [-0.5, 0.5, 0.],
            color,
            tex_coord: [0., 1.],
        },
        VertexAttributes {
            position: [-0.5, -0.5, 0.],
            color,
            tex_coord: [0., 0.],
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

#[derive(Debug)]
struct Texture {
    image: image::RgbaImage,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}
impl Texture {
    pub fn new(device: &wgpu::Device, bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes).unwrap().flipv();
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("wall"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let desc = wgpu::TextureViewDescriptor::default();
        let view = texture.create_view(&desc);
        let desc = wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        };
        let sampler = device.create_sampler(&desc);
        Self {
            image,
            texture,
            view,
            sampler,
        }
    }

    pub fn register(&self, queue: &wgpu::Queue) {
        let texture = wgpu::ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };
        let layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(texture.texture.size().width * 4),
            rows_per_image: None,
        };
        queue.write_texture(texture, &self.image, layout, texture.texture.size());
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
    pub fn texture_layout(&self) -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }
    pub fn sampler_layout(&self) -> wgpu::BindingType {
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }
}
