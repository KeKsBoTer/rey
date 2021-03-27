#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            }],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1., 0.0],
    }, // A
    Vertex {
        position: [1.0, 1.0, 0.0],
    }, // B
    Vertex {
        position: [-1.0, -1.0, 0.0],
    }, // C
    Vertex {
        position: [1.0, -1.0, 0.0],
    },
];

pub struct StorageTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl StorageTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let tv = StorageTexture::create(device, width, height);
        StorageTexture {
            texture: tv.0,
            view: tv.1,
        }
    }

    fn create(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::COPY_DST,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture.destroy();
        self.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::COPY_DST,
        });
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn destroy(&self) {
        self.texture.destroy();
    }
}

impl From<StorageTexture> for wgpu::BindingType {
    fn from(_item: StorageTexture) -> Self {
        wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::ReadOnly,
            /// Format of the texture.
            format: wgpu::TextureFormat::Rgba8Unorm,
            /// Dimension of the texture view that is going to be sampled.
            view_dimension: wgpu::TextureViewDimension::D2,
        }
    }
}

pub fn next_power_of_two(n: u32) -> u32 {
    let mut x = n;
    x -= 1;
    x |= x >> 1; // handle 2 bit numbers
    x |= x >> 2; // handle 4 bit numbers
    x |= x >> 4; // handle 8 bit numbers
    x |= x >> 8; // handle 16 bit numbers
    x |= x >> 16; // handle 32 bit numbers
    x += 1;
    return x;
}

pub fn create_texture<'a>(device: &'a wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    return device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Output Texture"),
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::STORAGE,
    });
}
