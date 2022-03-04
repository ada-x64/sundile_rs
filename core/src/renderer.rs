use wgpu;
use winit;
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use legion::*;
use std::collections::HashMap;

pub struct Renderer {
    pub viewport: Option<Viewport>,

    pub camera_wrapper: CameraWrapper,
    pub light_wrapper: LightWrapper,
    
	model_pipeline: wgpu::RenderPipeline,

    instance_cache_map: HashMap<String, InstanceCache>,
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
            
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Model Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &assets.shaders["default"],
                    entry_point: "vs_main",
                    buffers: &[
                        ModelVertex::desc(),
                        InstanceRaw::desc(),
                    ]
                },
                fragment: Some(wgpu::FragmentState {
                    module: &assets.shaders["default"],
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: config.format,
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
                    format: Texture::DEPTH_FORMAT,
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
        };

        //NOTE: Clone here because assets struct is dropped in game::new()
        let cache_iter = assets.models.iter().map(|(name, _)| {
            (name.to_owned(), InstanceCache::new())
        });
        let instance_cache_map = HashMap::from_iter(cache_iter);

        Renderer {
            viewport,
            
            camera_wrapper,
            light_wrapper,
    
            model_pipeline,
            instance_cache_map,
        }
    }    

	pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_wrapper.update(dt);
	}

    pub fn handle_input(&mut self, event: &winit::event::DeviceEvent) {
        self.camera_wrapper.handle_input(event);
    }
    
    pub fn render(&mut self, render_target: &mut RenderTarget, _world: &World, assets: &Assets) {
        //
        // Setup
        //
        self.camera_wrapper.render(&render_target.queue);
        let light_bind_group = self.light_wrapper.get_bind_group(&render_target.device);
        let camera_bind_group = &self.camera_wrapper.bind_group;

        for (_, cache) in &mut self.instance_cache_map {
            use cgmath::*;
            cache.clear();
            cache.insert(Instance { //TEMP
                position: Vector3::zero(),
                rotation: Quaternion::zero(),
            });
            cache.update(&render_target.device);
        }

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
        for (name, cache) in &mut self.instance_cache_map {
            cache.render(&mut render_pass, &assets.models[name], camera_bind_group, &light_bind_group);
        }
    }   
}