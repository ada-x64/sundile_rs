use std::sync::{Arc, Mutex};

use sundile_assets::*;
use sundile_common::*;
use sundile_graphics::*;

pub struct Renderer {
    pub viewport: Option<Viewport>,

    pub camera_wrapper: CameraWrapper,
    pub light_wrapper: LightWrapper,

    model_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(
        render_target: &RenderTarget,
        assets: &mut AssetTypeMap,
        viewport: Option<Viewport>,
    ) -> Self {
        //
        // Setup
        //
        let (device, config) = (&render_target.device, &render_target.config);

        let (width, height) = {
            if let Some(viewport) = viewport {
                (viewport.width as u32, viewport.height as u32)
            } else {
                (config.width as u32, config.height as u32)
            }
        };

        let camera_wrapper = CameraWrapper::new(&device, width, height);
        let mut light_wrapper = LightWrapper::new(&device);
        light_wrapper.set_ambient(Color::from_rgba(1.0, 1.0, 1.0, 0.1).as_array());
        light_wrapper
            .add_light(
                "test",
                LightUniform::new([0.0, 1.0, 0.0], [1.0, 1.0, 1.0, 1.0]),
            )
            .unwrap();

        //
        // Pipelines
        //

        let camera_bind_group_layout = &camera_wrapper.bind_group_layout;
        let light_bind_group_layout = &light_wrapper.bind_group_layout;
        let texture_bind_group_layout = &render_target.texture_layout;

        let model_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let default_shader = assets
            .try_get_asset::<wgpu::ShaderModule>("default")
            .unwrap();

        let model_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Model Pipeline"),
            layout: Some(&model_pipeline_layout),
            vertex: wgpu::VertexState {
                module: default_shader.as_ref(),
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: default_shader.as_ref(),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::DEPTH_FORMAT,
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
        });

        Renderer {
            viewport,

            camera_wrapper,
            light_wrapper,

            model_pipeline,
        }
    }

    pub fn update(&mut self, dt: sundile_common::time::Duration) {
        self.camera_wrapper.update(dt);

        use cgmath::*;
        let mut light = self.light_wrapper.get_light("test").unwrap();
        light.position = [
            light.position[0] + Angle::cos(Rad::<f32>(std::f32::consts::PI * dt.as_secs_f32())),
            light.position[1],
            light.position[2] + Angle::sin(Rad::<f32>(std::f32::consts::PI * dt.as_secs_f32())),
        ];
        self.light_wrapper.update_light("test", light).unwrap();
    }

    pub fn handle_input(&mut self, input: &Input) {
        self.camera_wrapper.handle_input(input);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget, assets: Arc<Mutex<AssetTypeMap>>) {
        //
        // Setup
        //
        self.camera_wrapper.render(&render_target.queue);
        let light_bind_group = self.light_wrapper.get_bind_group(&render_target.device);
        let camera_bind_group = &self.camera_wrapper.bind_group;

        let mut assets = assets.lock().unwrap();
        let mut model_map = assets.try_take_asset_map::<Model>().ok();
        if let Some(mm) = model_map.as_mut() {
            for (_, model) in mm {
                model.instance_cache.update(&render_target.device);
            }
        }

        //
        // Rendering
        // Note: render_target _cannot_ be borrowed again once render_pass has been created.
        // Ensure that all processing is done before this point.
        //

        {
            let mut render_pass = render_target.get_render_pass(true, true);

            if let Some(viewport) = self.viewport.as_ref() {
                render_pass.set_viewport(
                    viewport.x,
                    viewport.y,
                    viewport.width,
                    viewport.height,
                    viewport.min_depth,
                    viewport.max_depth,
                );
            }

            if let Some(mm) = model_map.as_ref() {
                render_pass.set_pipeline(&self.model_pipeline);
                for (_, model) in mm {
                    model.render(&mut render_pass, &camera_bind_group, &light_bind_group);
                }
            }
        }

        if let Some(mm) = model_map {
            assets.insert_map(mm);
        }
    }
}
