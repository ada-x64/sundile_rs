use sundile_assets::AssetTypeMap;
use sundile_graphics::{Font, RenderTarget, TextureWrapper};

use crate::SceneBuilder;

/// Default scene. Does nothing.
pub fn default_scene(_: SceneBuilder) {}

/// Loads in all default assets.
pub fn load_default_assets(render_target: &RenderTarget, assets: &mut AssetTypeMap) {
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
