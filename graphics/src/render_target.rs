use crate::*;

pub struct RenderTarget {
    pub adapter: wgpu::Adapter,
	pub config: wgpu::SurfaceConfiguration,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
	pub surface: wgpu::Surface,

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
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
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
                label: label,
            },
            trace_path,
        ).await.unwrap();

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

            surface_texture: None,
            encoder: None,
            color_view: None,
            depth_view: None,
        }
    }

    pub fn begin_frame(&mut self) {
        self.surface_texture = Some(
            self.surface.get_current_texture().expect("Unable to get surface texture!")
        );
        self.encoder = Some(
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Renderer Encoder"),
            })
        );
        self.color_view = Some(
            self.surface_texture.as_ref().unwrap().texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("Renderer Texture View"),
                format: Some(self.surface.get_preferred_format(&self.adapter).unwrap()),
                ..Default::default()
            })
        );
        self.depth_view = Some(
            texture::Texture::create_depth_texture(&self.device, &self.config, "Depth Texture").view
        );
    }

    pub fn get_render_pass(&mut self, clear: bool, use_depth_stencil: bool,) -> wgpu::RenderPass {
       self.encoder.as_mut().unwrap().begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Renderer Pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.color_view.as_ref().unwrap(),
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
        self.queue.submit(std::iter::once(self.encoder.take().unwrap().finish()));
        self.surface_texture.take().unwrap().present();
    }
}
