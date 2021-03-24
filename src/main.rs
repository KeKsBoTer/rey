use core::f32;
use std::iter;

use std::time::{SystemTime};
use cgmath::prelude::*;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window},
};

mod pipeline;
mod lib;
mod camera;
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}


impl Uniforms {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    // UPDATED!
    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_proj = (camera.calc_matrix()).into() // TODO add perspective (ratio usw.)
    }
}


struct State {

    timestamp: SystemTime,

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

    vertex_buffer: wgpu::Buffer,
    render_bind_group: wgpu::BindGroup,

    comp_in_buffer: wgpu::Buffer,
    comp_bind_group: wgpu::BindGroup,
    render_bind_layout: wgpu::BindGroupLayout,
    compute_bind_layout: wgpu::BindGroupLayout,

    mouse_pressed: bool,
}

impl State {
    async fn new(window: &Window) -> Self {
        let mut size = window.inner_size();
        size = winit::dpi::PhysicalSize::new(size.width/2, size.height/2);

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
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
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);


        // UPDATED!
        let camera = camera::Camera::new((0.0, 0.0, 0.0), cgmath::Deg(0.0), cgmath::Deg(0.0),cgmath::Deg(0.0));
        let projection =
            camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(45.0));
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);


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
                label: Some("uniform_bind_group_layout"),
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
                        format: wgpu::TextureFormat::Rgba8Unorm,
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

        let render_pipeline = pipeline::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            sc_desc.format,
            &[lib::Vertex::desc()],
            wgpu::include_spirv!("shader.vert.spv"),
            wgpu::include_spirv!("shader.frag.spv"),
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(lib::VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let compute_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Binder"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,                             // The location
                    visibility: wgpu::ShaderStage::COMPUTE, // Which shader type in the pipeline this buffer is available to.
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            // Specifies if the buffer can only be read within the shader
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        /// Format of the texture.
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        /// Dimension of the texture view that is going to be sampled.
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let compute_pipeline = pipeline::create_compute_pipeline(
            &device,
            &[
                &compute_bind_layout,
                &uniform_bind_group_layout,
            ],
            wgpu::include_spirv!("shader.comp.spv"),
            Some("ComputePipeline"),
        );

        let numbers: [f32; 3] = [0.5, 0., 0.];

        let comp_in_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(&numbers),
            usage: wgpu::BufferUsage::STORAGE,
        });

        let out_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::STORAGE,
        });

        let view = out_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Instantiates the bind group, once again specifying the binding of buffers.
        let comp_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: comp_in_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });

        // Instantiates the bind group, once again specifying the binding of buffers.
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &render_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        let timestamp = SystemTime::now();

        Self {
            camera,
            projection,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,

            timestamp,
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            compute_pipeline,

            vertex_buffer,
            render_bind_group,

            comp_in_buffer,
            comp_bind_group,

            render_bind_layout,
            compute_bind_layout,

            mouse_pressed: false,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = winit::dpi::PhysicalSize::new(new_size.width/2, new_size.height/2);
        self.sc_desc.width = self.size.width;
        self.sc_desc.height = self.size.width;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.projection.resize(self.size.width,self.size.height);

        // recreate screen texture

        let out_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::STORAGE,
        });

        let view = out_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Instantiates the bind group, once again specifying the binding of buffers.
        self.comp_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.compute_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.comp_in_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });

        self.render_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.render_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });


        println!("{:} {:}",self.size.width, self.size.height);
    }


    // UPDATED!
    fn input(&mut self, event: &DeviceEvent) -> bool {
        //println!("{:?}",event);
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => {
                self.camera_controller.process_keyboard(*key, *state)
            },
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button:0, // Left Mouse Button
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
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
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniforms
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        //println!("{:?}",self.uniforms.view_proj);
        //println!("{:?}",self.camera.position);
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {

        let current = SystemTime::now();
        let delta = current.duration_since(self.timestamp).unwrap().as_millis();
        //println!("{:?} FPS", 1000/delta);

        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut c_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            c_pass.set_pipeline(&self.compute_pipeline);
            c_pass.set_bind_group(0, &self.comp_bind_group, &[]);
            c_pass.set_bind_group(1, &self.uniform_bind_group,&[]);
            c_pass.insert_debug_marker("compute stuff");
            c_pass.dispatch(self.size.width, self.size.height, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..lib::VERTICES.len() as u32, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));

        self.timestamp = current;

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
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                global_state.input(event);
            }
            // UPDATED!
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
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
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        global_state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        global_state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                global_state.update(dt);
                match global_state.render() {
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