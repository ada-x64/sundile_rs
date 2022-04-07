use crate::*;

fn get_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
            ],
            label: Some("texture_bind_group_layout"),
        }
    )
}

pub struct HeadlessRenderTarget {
    pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub texture_layout: wgpu::BindGroupLayout,
}
impl HeadlessRenderTarget {
    pub async fn new(enable_tracing:bool, label: Option<&str>) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let mut trace_path = None;
        let dir = format!("./dbg/trace/{}__{}", chrono::Local::now().timestamp(), label.unwrap_or_else(|| "UNLABLED"));
        let path = std::path::Path::new(&*dir);
        if enable_tracing {
            std::fs::create_dir_all(&path).expect("Unable to create tracing path!");
            trace_path = Some(path);
        }

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                limits: wgpu::Limits::default(),
                label,
            },
            trace_path,
        ).await.unwrap();
        
        let texture_layout = get_texture_bind_group_layout(&device);

        Self {
            adapter,
            device,
            queue,
            instance,
            texture_layout,
        }
    }
}


pub struct RenderTarget {
    pub adapter: wgpu::Adapter,
	pub config: wgpu::SurfaceConfiguration,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
	pub surface: wgpu::Surface,
    pub texture_layout: wgpu::BindGroupLayout,

    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub encoder: Option<wgpu::CommandEncoder>,
    pub color_view: Option<wgpu::TextureView>,
    pub depth_view: Option<wgpu::TextureView>,
}
impl RenderTarget {
    pub async fn new(window: &winit::window::Window, enable_tracing: bool, label: Option<&str>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.expect("Failed to create adapter!");

        let mut trace_path = None;
        let dir = format!("./dbg/trace/{}__{}", chrono::Local::now().format("%F-%s"), label.unwrap_or_else(|| "UNLABLED"));
        let path = std::path::Path::new(&*dir);
        if enable_tracing {
            use log::debug;
            std::fs::create_dir_all(&path).expect("Unable to create tracing path!");
            trace_path = Some(path);
            debug!("Render target tracing enabled.");
        }

        #[cfg(target_arch="wasm32")]
        let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        #[cfg(not(target_arch="wasm32"))]
        let limits = wgpu::Limits::default();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits,
                label,
            },
            trace_path,
        ).await.unwrap();

        let texture_layout = get_texture_bind_group_layout(&device);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);
        
        Self {
            adapter,
            config,
            device,
            queue,
            instance,
            surface,
            texture_layout,

            surface_texture: None,
            encoder: None,
            color_view: None,
            depth_view: None,
        }
    }

    pub fn begin_frame(&mut self) {
        if self.surface_texture.is_none() {
            self.surface_texture = Some(
                self.surface.get_current_texture().expect("Unable to get surface texture!")
            );
            self.encoder = Some(
                self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Target Encoder"),
                })
            );
            self.color_view = Some(
                self.surface_texture.as_ref().unwrap().texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Render Target Texture View"),
                    format: Some(self.surface.get_preferred_format(&self.adapter).unwrap()),
                    ..Default::default()
                })
            );
            self.depth_view = Some(
                texture::TextureWrapper::create_depth_texture(&self.device, &self.config, "Depth Texture").view
            );
        }
    }

    pub fn get_render_pass(&mut self, clear: bool, use_depth_stencil: bool,) -> wgpu::RenderPass {
        self.encoder.as_mut().unwrap().begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: self.color_view.as_ref().unwrap(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if clear { wgpu::LoadOp::Clear(wgpu::Color {r: 1.0, g: 0.0, b: 1.0, a: 0.0}) }
                            else { wgpu::LoadOp::Load },
                        store: true,
                    },
                }],
                depth_stencil_attachment: 
                    if use_depth_stencil {
                        Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_view.as_ref().unwrap(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        })
                    }
                    else {None},
            })
    }

    pub fn end_frame(&mut self) {
        if self.surface_texture.is_some() {
            self.queue.submit(std::iter::once(self.encoder.take().unwrap().finish()));
            self.surface_texture.take().unwrap().present();
        }
    }
}
