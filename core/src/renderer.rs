use wgpu;
use winit;
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use legion::*;

pub fn create_render_pipeline(
	name: &str,
	device: &wgpu::Device,
	layout: Option<&wgpu::PipelineLayout>,
	color_format: wgpu::TextureFormat,
	depth_format: wgpu::TextureFormat,
	vertex_layouts: &[wgpu::VertexBufferLayout],
	vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
	device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some(name),
		layout: layout,
		vertex: wgpu::VertexState {
			module: &vertex_shader,
			entry_point: "vs_main",
			buffers: vertex_layouts
		},
		fragment: Some(wgpu::FragmentState {
			module: &fragment_shader,
			entry_point: "fs_main",
			targets: &[wgpu::ColorTargetState {
				format: color_format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			}],
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false
		},
		depth_stencil: Some(wgpu::DepthStencilState {
			format: depth_format,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
	})
}

pub struct Renderer {
    pub viewport: Option<Viewport>,

    pub camera_wrapper: CameraWrapper,
    pub light_wrapper: LightWrapper,
    
	model_pipeline: wgpu::RenderPipeline,
}

impl Renderer {

    pub fn new(render_target: &RenderTarget, assets: &Assets, viewport: Option<Viewport>) -> Self {
        //
        // Setup
        //
        let (device, config, ) = (
            &render_target.device,
            &render_target.config,
        );

        let (width, height) = {
            if viewport.is_some() {let vp = viewport.as_ref().unwrap(); (vp.width as u32, vp.height as u32)}
            else {(config.width as u32, config.height as u32)}
        };

        let camera_wrapper = CameraWrapper::new(&device, width, height);
        let light_wrapper = LightWrapper::new(&device,);
    
        //
        // Pipelines
        //

        let camera_bind_group_layout = &camera_wrapper.bind_group_layout;
        let light_bind_group_layout = &light_wrapper.bind_group_layout;
        let texture_bind_group_layout = Texture::get_bind_group_layout(device);

        let model_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                    ],
                push_constant_ranges: &[],
            });
            create_render_pipeline(
                "Model Pipeline",
                &device,
                Some(&layout),
                config.format,
                Texture::DEPTH_FORMAT,
                &[ModelVertex::desc(), InstanceRaw::desc()],
                &assets.shaders["default"],
                &assets.shaders["default"],
            )
        };

        Renderer {
            viewport,
            
            camera_wrapper,
            light_wrapper,
    
            model_pipeline,
        }
    }    

	pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_wrapper.update(dt);
	}

    pub fn handle_input(&mut self, event: &winit::event::DeviceEvent) {
        self.camera_wrapper.handle_input(event);
    }
    
    pub fn render(&mut self, render_target: &mut RenderTarget, world: &World) {
        //
        // Setup
        //
        self.camera_wrapper.render(&render_target.queue);
        let light_bind_group = self.light_wrapper.get_bind_group(&render_target.device);

        // 
        // Rendering
        // Note: render_target _cannot_ be borrowed again once render_pass has been created.
        // Ensure that all processing is done before this point.
        //
        let mut render_pass = render_target.get_render_pass(true, true);

        if self.viewport.is_some() {
            let viewport = self.viewport.as_ref().unwrap();
            render_pass.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height, viewport.min_depth, viewport.max_depth);
        }

        render_pass.set_pipeline(&self.model_pipeline);
        Read::<&Model>::query().for_each(world, |model| {
            render_pass.draw_model(model, &self.camera_wrapper.bind_group, &light_bind_group);
        });
    }   
}