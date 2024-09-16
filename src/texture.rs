use crate::WndSize;

#[derive(Debug)]
pub struct ImageTexture {
    image: image::RgbaImage,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}
impl ImageTexture {
    pub fn new(device: &wgpu::Device, bytes: &[u8], label: Option<&str>) -> Self {
        let image = image::load_from_memory(bytes).unwrap().flipv();
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label,
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
        Self {
            image,
            texture,
            view,
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
    pub fn texture_layout(&self) -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }
}

#[derive(Debug)]
pub struct ImageSampler {
    sampler: wgpu::Sampler,
}
impl ImageSampler {
    pub fn new(device: &wgpu::Device, label: Option<&str>) -> Self {
        let desc = wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        };
        let sampler = device.create_sampler(&desc);
        Self { sampler }
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
    pub fn sampler_layout(&self) -> wgpu::BindingType {
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}
impl DepthBuffer {
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(device: &wgpu::Device, size: WndSize, label: Option<&str>) -> Self {
        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let desc = wgpu::TextureViewDescriptor::default();
        let view = texture.create_view(&desc);
        Self {
            _texture: texture,
            view,
        }
    }

    pub fn state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: Self::FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }
    }

    pub fn attachment_clear(&self) -> wgpu::RenderPassDepthStencilAttachment {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
    pub fn attachment_load(&self) -> wgpu::RenderPassDepthStencilAttachment {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
}
