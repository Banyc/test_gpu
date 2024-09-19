use std::{
    f64::consts::PI,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use bytemuck_derive::{Pod, Zeroable};
use num_traits::Float;
use wgpu::util::DeviceExt;

use crate::{
    camera::{Camera, Heave, RotationalMovement, Surge, Sway, TranslationalMovement},
    delta_time::DeltaTime,
    texture::{DepthBuffer, ImageSampler, ImageTexture},
    transform::{perspective, rotate, translate},
    Draw, DrawArgs, RenderApp, RenderContext, RenderInit, RenderInitArgs, RenderNextStep, Resize,
    ResizeArgs, Update, UpdateArgs, WndSize,
};

const SHADER: &str = include_str!("triangle.wgsl");
const WALL: &[u8] = include_bytes!("wall.jpg");
const IS_WIREFRAME: bool = false;
const SIN_WAVE_X_PER_PERIOD: usize = 2 << 10;

#[derive(Debug)]
pub struct DrawTriangleInit {}
impl DrawTriangleInit {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for DrawTriangleInit {
    fn default() -> Self {
        Self::new()
    }
}
impl RenderInit for DrawTriangleInit {
    fn init(&self, args: RenderInitArgs<'_>) -> Box<dyn RenderApp> {
        Box::new(DrawTriangle::new(args))
    }
}

#[derive(Debug)]
struct DrawTriangle {
    wnd_size: WndSize,
    depth_buffer: DepthBuffer,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    camera: Camera,
    draw_delta_time: DeltaTime,
}
impl DrawTriangle {
    pub fn new(args: RenderInitArgs<'_>) -> Self {
        let texture = ImageTexture::new(args.device, WALL, Some("wall"));
        texture.register(args.queue);
        let sampler = ImageSampler::new(args.device, Some("sampler"));
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
        // let mesh = triangle();
        // let mesh = rectangle();
        let mesh = cube();
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
                    ty: sampler.sampler_layout(),
                    count: None,
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler.sampler()),
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
            depth_stencil: Some(DepthBuffer::state()),
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
        let depth_buffer = DepthBuffer::new(args.device, args.wnd_size, Some("depth buffer"));
        let camera = Camera::new();
        let draw_delta_time = DeltaTime::new(Instant::now());
        Self {
            wnd_size: args.wnd_size,
            depth_buffer,
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            uniform_buffer,
            bind_group,
            camera,
            draw_delta_time,
        }
    }

    fn update_camera(&mut self, context: &RenderContext) {
        let Some(delta_time) = self.draw_delta_time.delta() else {
            return;
        };
        let is_w = context.input.is_key_pressed(winit::keyboard::KeyCode::KeyW);
        let is_s = context.input.is_key_pressed(winit::keyboard::KeyCode::KeyS);
        let surge = match (is_w, is_s) {
            (true, true) | (false, false) => None,
            (true, false) => Some(Surge::Forward),
            (false, true) => Some(Surge::Backward),
        };
        let is_a = context.input.is_key_pressed(winit::keyboard::KeyCode::KeyA);
        let is_d = context.input.is_key_pressed(winit::keyboard::KeyCode::KeyD);
        let sway = match (is_a, is_d) {
            (true, true) | (false, false) => None,
            (true, false) => Some(Sway::Left),
            (false, true) => Some(Sway::Right),
        };
        let is_space = context
            .input
            .is_key_pressed(winit::keyboard::KeyCode::Space);
        let is_shift_left = context
            .input
            .is_key_pressed(winit::keyboard::KeyCode::ShiftLeft);
        let heave = match (is_space, is_shift_left) {
            (true, true) | (false, false) => None,
            (true, false) => Some(Heave::Up),
            (false, true) => Some(Heave::Down),
        };
        let movement = TranslationalMovement { surge, sway, heave };
        self.camera.translate(movement, delta_time.as_secs_f64());
    }
}
impl Draw for DrawTriangle {
    fn draw(&mut self, args: DrawArgs<'_>) -> RenderNextStep {
        self.draw_delta_time.update(Instant::now());
        self.update_camera(args.context);
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
        let normalized_sin = normalized_sin();
        dbg!(normalized_sin);
        // let trans = {
        //     let translate = translate([0.5, -0.5, 0.0]);
        //     let angle = sin * PI * 2.;
        //     let rotate = rotate([0.0, 0.0, 1.0], angle);
        //     matrix_mul(&rotate, &translate)
        // };
        // dbg!(&trans);

        let desc = wgpu::CommandEncoderDescriptor {
            label: Some("clear"),
        };
        let mut command = args.device.create_command_encoder(&desc);
        {
            let desc = wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(background.clone())],
                depth_stencil_attachment: Some(self.depth_buffer.attachment_clear()),
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let _ = command.begin_render_pass(&desc);
        }
        args.queue.submit([command.finish()]);

        // let radius = 10.;
        // let (sin, cos) = waves();
        // let view = look_at([sin * radius, 0., cos * radius], [0., 0., 0.], [0., 1., 0.]);
        let view = self.camera.view_matrix();
        let aspect = self.wnd_size.width as f64 / self.wnd_size.height as f64;
        let projection = perspective(self.camera.fov(), aspect, 0.1, 100.);

        let model_positions = [
            [0.0, 0.0, 0.0],
            [2.0, 5.0, -15.0],
            [-1.5, -2.2, -2.5],
            [-3.8, -2.0, -12.3],
            [2.4, -0.4, -3.5],
            [-1.7, 3.0, -7.5],
            [1.3, -2.0, -2.5],
            [1.5, 2.0, -2.5],
            [1.5, 0.2, -1.5],
            [-1.3, 1.0, -1.5],
        ];
        let models = model_positions.into_iter().map(translate);
        for (i, model_position) in models.enumerate() {
            let rotate = rotate([1., 0.3, 0.5], normalized_sin * i as f64 * 20. * PI / 180.);
            let model = model_position.mul_matrix_square(&rotate);
            let uniform = Uniform {
                model: model.transpose().into_buffer().map(|x| x as f32),
                view: view.transpose().into_buffer().map(|x| x as f32),
                projection: projection.transpose().into_buffer().map(|x| x as f32),
                _padding: [0; 3],
                sin: normalized_sin as f32,
            };
            args.queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));
            let desc = wgpu::CommandEncoderDescriptor { label: None };
            let mut command = args.device.create_command_encoder(&desc);
            {
                let color = wgpu::RenderPassColorAttachment {
                    view: &args.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                };
                let desc = wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(color)],
                    depth_stencil_attachment: Some(self.depth_buffer.attachment_load()),
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

        RenderNextStep {
            should_request_redraw: true,
        }
    }
}
impl Update for DrawTriangle {
    fn update(&mut self, args: UpdateArgs) -> RenderNextStep {
        if let winit::event::WindowEvent::MouseWheel {
            device_id: _,
            delta,
            phase: _,
        } = &args.event
        {
            let y = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y as f64,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y,
            };
            let scale_to_radian = (2.0_f64).powi(7);
            self.camera.zoom(-y / scale_to_radian);
        }
        if let Some(cursor_change) = args.context.input.cursor_change() {
            let scale_to_radian = (2.0_f64).powi(4);
            let movement = RotationalMovement {
                yaw: -cursor_change.x / scale_to_radian,
                pitch: cursor_change.y / scale_to_radian,
            };
            self.camera.rotate(movement);
        }
        RenderNextStep {
            should_request_redraw: false,
        }
    }
}
impl Resize for DrawTriangle {
    fn resize(&mut self, args: ResizeArgs<'_>) -> RenderNextStep {
        self.wnd_size = args.size;
        self.depth_buffer = DepthBuffer::new(args.device, args.size, Some("depth buffer"));
        RenderNextStep {
            should_request_redraw: false,
        }
    }
}
impl RenderApp for DrawTriangle {}

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
    let indices = vertex_pos.indices().into();
    Mesh { vertices, indices }
}

#[allow(unused)]
fn cube() -> Mesh {
    let color = [1.0, 0.5, 0.2];
    let vertices = vec![
        VertexAttributes {
            position: [-0.5, -0.5, -0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, -0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, -0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, -0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, -0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, 0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, 0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, 0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, 0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, 0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, -0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, 0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, -0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, 0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, -0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, -0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, 0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, -0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, -0.5],
            tex_coord: [1.0, 1.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [0.5, 0.5, 0.5],
            tex_coord: [1.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, 0.5],
            tex_coord: [0.0, 0.0],
            color,
        },
        VertexAttributes {
            position: [-0.5, 0.5, -0.5],
            tex_coord: [0.0, 1.0],
            color,
        },
    ];
    let indices = vertices
        .iter()
        .enumerate()
        .map(|(i, _)| i as u32)
        .collect::<Vec<u32>>();
    Mesh { vertices, indices }
}

#[derive(Debug, Clone, Copy)]
struct QuadVertexPos {
    pub top_right: u32,
    pub bottom_right: u32,
    pub top_left: u32,
    pub bottom_left: u32,
}
impl QuadVertexPos {
    pub fn indices(&self) -> [u32; 6] {
        [
            self.top_right,
            self.bottom_right,
            self.top_left, //
            self.bottom_right,
            self.bottom_left,
            self.top_left,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
struct CubeVertexPos {
    pub top: QuadVertexPos,
    pub bottom: QuadVertexPos,
    pub left: QuadVertexPos,
    pub right: QuadVertexPos,
    pub front: QuadVertexPos,
    pub behind: QuadVertexPos,
}
impl CubeVertexPos {
    pub fn indices(&self) -> [u32; 36] {
        self.top
            .indices()
            .into_iter()
            .chain(self.bottom.indices())
            .chain(self.left.indices())
            .chain(self.right.indices())
            .chain(self.front.indices())
            .chain(self.behind.indices())
            .collect::<Vec<u32>>()
            .try_into()
            .unwrap()
    }
}

fn normalized_sin() -> f64 {
    let (sin, _) = waves();
    normalize_neg_pos_1(sin)
}
fn normalize_neg_pos_1<T: Float>(v: T) -> T {
    let one = T::from(1.).unwrap();
    let two = T::from(2.).unwrap();
    (v + one) / two
}

fn waves() -> (f64, f64) {
    let x = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % (SIN_WAVE_X_PER_PERIOD as u128)) as f64
        * 2.
        * PI
        / SIN_WAVE_X_PER_PERIOD as f64;
    (x.sin(), x.cos())
}
