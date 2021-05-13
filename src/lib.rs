use wgpu::{util::DeviceExt, Texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
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

pub fn create_texture<'a>(
    device: &'a wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsage,
) -> wgpu::Texture {
    return device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Output Texture"),
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: format,
        usage: wgpu::TextureUsage::STORAGE | usage,
    });
}

pub fn create_empty_texture<'a>(
    device: &'a wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsage,
) -> wgpu::Texture {
    let data = vec![0u8; (width * height * (format.describe().block_size as u32)) as usize];
    let d = data.as_slice();
    return device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format,
            usage: wgpu::TextureUsage::STORAGE | usage,
        },
        d,
    );
}

pub struct FrameBuffer {
    pub src: wgpu::Texture,
    pub dst: wgpu::Texture,
    pub width: u32,
    pub height: u32,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let (src, dst) = create_frame_buffer_textures(width, height, device, queue);
        Self {
            src,
            dst,
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.src.destroy();
        self.dst.destroy();
        let (src, dst) = create_frame_buffer_textures(width, height, device, queue);
        self.src = src;
        self.dst = dst;
        self.width = width;
        self.height = height;
    }

    pub fn create_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        let frame_buffer_src_view = self
            .src
            .create_view(&wgpu::TextureViewDescriptor::default());
        let frame_buffer_dst_view = self
            .dst
            .create_view(&wgpu::TextureViewDescriptor::default());
        (frame_buffer_src_view, frame_buffer_dst_view)
    }
}

fn create_frame_buffer_textures(
    width: u32,
    height: u32,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (Texture, Texture) {
    let src = create_empty_texture(
        device,
        queue,
        width,
        height,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsage::COPY_DST,
    );
    let dst = create_texture(
        device,
        width,
        height,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsage::COPY_SRC,
    );
    (src, dst)
}
