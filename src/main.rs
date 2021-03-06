use core::f32;
use std::iter;

use cgmath::{prelude::*, Vector2};
use std::borrow::Cow;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod camera;
mod cornell_box;
mod lib;
mod pipeline;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    time: f32,
    pass: u32,
    num_faces: u32,
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            time: 0.0,
            pass: 0,
            num_faces: 0,
        }
    }

    fn increment_time(&mut self, dt: f32) {
        self.time += dt;
    }

    fn increment_pass(&mut self) {
        self.pass += 1;
    }

    fn reset_pass(&mut self) {
        self.pass = 0;
    }

    // UPDATED!
    fn update_view_proj(&mut self, camera: &camera::Camera, _projection: &camera::Projection) {
        self.view_proj = (camera.calc_matrix()).into() // TODO add perspective (ratio usw.)
    }
}
struct State {
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,

    frame_buffer: lib::FrameBuffer,

    render_bind_layout: wgpu::BindGroupLayout,
    render_bind_group: wgpu::BindGroup,

    framebuffer_bind_group_layout: wgpu::BindGroupLayout,
    framebuffer_bind_group: wgpu::BindGroup,

    vertex_bind_group: wgpu::BindGroup,

    mouse_pressed: bool,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let camera = camera::Camera::new(
            (250.0, 350.0, -260.0),
            cgmath::Deg(0.0),
            cgmath::Deg(0.0),
            cgmath::Deg(0.0),
        );
        let projection = camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(45.0));
        let camera_controller = camera::CameraController::new(400.0, 0.4);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);
        uniforms.num_faces = cornell_box::FACES.len() as u32;

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let render_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render Binder"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,                              // The location
                    visibility: wgpu::ShaderStage::FRAGMENT, // Which shader type in the pipeline this buffer is available to.
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        /// Format of the texture.
                        format: wgpu::TextureFormat::Rgba16Float,
                        /// Dimension of the texture view that is going to be sampled.
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_layout],
                push_constant_ranges: &[],
            });

        let flags = wgpu::ShaderFlags::VALIDATION;

        let render_pipeline = pipeline::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            sc_desc.format,
            wgpu::ShaderModuleDescriptor {
                label: Some("display_shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("display.wgsl"))),
                flags,
            },
        );

        let framebuffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Binder"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            /// Format of the texture.
                            format: wgpu::TextureFormat::Rgba16Float,
                            /// Dimension of the texture view that is going to be sampled.
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            /// Format of the texture.
                            format: wgpu::TextureFormat::Rgba16Float,
                            /// Dimension of the texture view that is going to be sampled.
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let frame_buffer = lib::FrameBuffer::new(size.width, size.height, &device, &queue);

        let (fb_src_view, fb_dst_view) = frame_buffer.create_views();

        // Instantiates the bind group, once again specifying the binding of buffers.
        let framebuffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &framebuffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&fb_src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&fb_dst_view),
                },
            ],
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&cornell_box::VERTICES),
            usage: wgpu::BufferUsage::STORAGE,
        });

        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("face buffer"),
            contents: bytemuck::cast_slice(&cornell_box::FACES),
            usage: wgpu::BufferUsage::STORAGE,
        });

        let vertex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("vertex bind layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vertex bind group"),
            layout: &vertex_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: face_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline = pipeline::create_compute_pipeline(
            &device,
            &[
                &framebuffer_bind_group_layout,
                &uniform_bind_group_layout,
                &vertex_bind_group_layout,
            ],
            wgpu::ShaderModuleDescriptor {
                label: Some("display_shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
                flags,
            },
            Some("ComputePipeline"),
        );

        // Instantiates the bind group, once again specifying the binding of buffers.
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &render_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&fb_dst_view),
            }],
        });

        Self {
            camera,
            projection,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,

            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            compute_pipeline,

            frame_buffer,

            render_bind_layout,
            render_bind_group,

            framebuffer_bind_group_layout,
            framebuffer_bind_group,

            vertex_bind_group,

            mouse_pressed: false,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = self.size.width;
        self.sc_desc.height = self.size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.projection.resize(self.size.width, self.size.height);

        self.frame_buffer
            .resize(self.size.width, self.size.height, &self.device, &self.queue);

        let (fb_src_view, fb_dst_view) = self.frame_buffer.create_views();

        // Instantiates the bind group, once again specifying the binding of buffers.
        self.framebuffer_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.framebuffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&fb_src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&fb_dst_view),
                },
            ],
        });

        self.render_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.render_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&fb_dst_view),
            }],
        });

        self.uniforms.reset_pass();

        println!("{:} {:}", self.size.width, self.size.height);
    }

    // UPDATED!
    fn input(&mut self, event: &DeviceEvent) -> bool {
        //println!("{:?}",event);
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.camera_controller.process_keyboard(*key, *state),
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        // UPDATED!
        let before = self.camera.calc_matrix();
        self.camera_controller.update_camera(&mut self.camera, dt);
        if self.camera.calc_matrix() != before {
            self.uniforms.reset_pass();
        }
        self.uniforms
            .increment_time((dt.as_millis() as f32) / 1000.);
        self.uniforms
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    fn render(&mut self, _dt: std::time::Duration) -> Result<(), wgpu::SwapChainError> {
        //println!("{:} FPS",1000/(dt.as_millis()+1));
        let frame = self.swap_chain.get_current_frame()?.output;

        let group_size = Vector2::new(32, 16);
        let width_groups = lib::next_power_of_two(self.size.width / group_size.x);
        let height_groups = lib::next_power_of_two(self.size.height / group_size.y);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut c_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            c_pass.set_pipeline(&self.compute_pipeline);
            c_pass.set_bind_group(0, &self.framebuffer_bind_group, &[]);
            c_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            c_pass.set_bind_group(2, &self.vertex_bind_group, &[]);
            c_pass.insert_debug_marker("compute stuff");
            // TODO use dispatch_indirect
            c_pass.dispatch(width_groups, height_groups, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            // TODO use draw_indirect
            render_pass.draw(0..6, 0..1);
        }

        // copy last frame to (currently in dst buffer) to src buffer
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &self.frame_buffer.dst,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyTexture {
                texture: &self.frame_buffer.src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::Extent3d {
                width: self.frame_buffer.width,
                height: self.frame_buffer.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(iter::once(encoder.finish()));

        self.uniforms.increment_pass();

        self.device.poll(wgpu::Maintain::Wait);

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    use futures::executor::block_on;
    let mut global_state = block_on(State::new(&window)); // NEW!
    let mut last_render_time = std::time::Instant::now();
    let mut last_pos: (f64, f64) = (0., 0.);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            // UPDATED!
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Destroyed => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    keyboard_input => {
                        global_state.input(&DeviceEvent::Key(*keyboard_input));
                    }
                },
                WindowEvent::CursorMoved { position, .. } => {
                    global_state.input(&DeviceEvent::MouseMotion {
                        delta: (position.x - last_pos.0, position.y - last_pos.1),
                    });
                    last_pos = (position.x, position.y);
                }
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Left,
                    ..
                } => {
                    global_state.mouse_pressed = *state == ElementState::Pressed;
                }
                WindowEvent::Resized(physical_size) => {
                    global_state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    global_state.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                global_state.update(dt);
                match global_state.render(dt) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => global_state.resize(global_state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });
}
