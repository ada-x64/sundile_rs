use sundile_assets::*;
use sundile_common::*;
use sundile_graphics::*;

use crate::renderer::*;
use crate::renderer2d::*;
use crate::SceneMap;

pub struct Game<'a> {
    pub renderer: Renderer<'a>,
    pub renderer2d: Renderer2d<'a>,
    pub paused: bool,
    pub assets: AssetTypeMap,
    scenes: SceneMap, //TODO: Possibly move this outside of Game struct so DebugGui has ability to change scenes?
}

fn load_default_assets(render_target: &RenderTarget, assets: &mut AssetTypeMap) {
    use log::info;
    use wgpu::*;

    // Shaders
    if assets.try_get_asset::<ShaderModule>("default").is_err() {
        let asset = render_target
            .device
            .create_shader_module(include_wgsl!("../assets/shaders/default.wgsl"));
        assets.try_insert_asset("default", asset).unwrap();
    } else {
        info!("Default shader overriden!");
    }
    if assets.try_get_asset::<ShaderModule>("2d").is_err() {
        let asset = render_target
            .device
            .create_shader_module(include_wgsl!("../assets/shaders/2d.wgsl"));
        assets.try_insert_asset("2d", asset).unwrap();
    } else {
        info!("Default 2d shader overriden!");
    }
    if assets.try_get_asset::<ShaderModule>("passthrough").is_err() {
        let asset = render_target
            .device
            .create_shader_module(include_wgsl!("../assets/shaders/passthrough.wgsl"));
        assets.try_insert_asset("passthrough", asset).unwrap();
    } else {
        info!("Passthrough shader overriden!");
    }

    // Fonts
    if assets.try_get_asset::<Font>("regular").is_err() {
        assets
            .try_insert_asset(
                "regular",
                Font {
                    data: include_bytes!("../assets/fonts/UBUNTUMONO-R.TTF").to_vec(),
                },
            )
            .unwrap();
    } else {
        info!("Default regular font overriden!");
    }
    if assets.try_get_asset::<Font>("italic").is_err() {
        assets
            .try_insert_asset(
                "italic",
                Font {
                    data: include_bytes!("../assets/fonts/UBUNTUMONO-RI.TTF").to_vec(),
                },
            )
            .unwrap();
    } else {
        info!("Default italic font overriden!");
    }
    if assets.try_get_asset::<Font>("bold").is_err() {
        assets
            .try_insert_asset(
                "bold",
                Font {
                    data: include_bytes!("../assets/fonts/UBUNTUMONO-B.TTF").to_vec(),
                },
            )
            .unwrap();
    } else {
        info!("Default bold font overriden!");
    }
    if assets.try_get_asset::<Font>("oblique").is_err() {
        assets
            .try_insert_asset(
                "oblique",
                Font {
                    data: include_bytes!("../assets/fonts/UBUNTUMONO-BI.TTF").to_vec(),
                },
            )
            .unwrap();
    } else {
        info!("Default oblique font overriden!");
    }

    assets
        .try_insert_asset(
            "test_atlas",
            TextureWrapper::from_bytes(
                &render_target.device,
                &render_target.queue,
                include_bytes!("../assets/textures/test_atlas.png"),
                "test atlas",
                false,
            )
            .unwrap(),
        )
        .unwrap();
}

impl<'a> Game<'a> {
    pub fn new(
        render_target: &RenderTarget,
        mut assets: AssetTypeMap,
        scenes: SceneMap,
        viewport: Option<Viewport>,
        paused: bool,
    ) -> Self {
        load_default_assets(render_target, &mut assets);

        let renderer = Renderer::new(&render_target, &mut assets, viewport);
        let renderer2d = Renderer2d::new(&render_target, &mut assets);

        Game {
            renderer,
            renderer2d,
            paused,
            assets,
            scenes,
        }
    }

    pub fn update(&mut self, dt: time::Time) {
        if self.paused {
            return;
        }
        self.renderer.update(dt);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget) {
        if self.paused {
            return;
        }
        self.renderer.render(render_target, &mut self.assets);
        self.renderer2d.render(render_target);
    }

    pub fn handle_input(&mut self, input: &Input) {
        if self.paused {
            return;
        }
        self.renderer.handle_input(input);
    }

    //TODO: Scenes should be assets loaded with AssetManager struct.
    pub fn set_scene<'s>(&mut self, scene: &'s str) {
        self.scenes[scene]();
    }
}
