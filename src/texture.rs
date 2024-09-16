#[derive(Debug)]
pub struct ImageTexture {
    image: image::RgbaImage,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
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
