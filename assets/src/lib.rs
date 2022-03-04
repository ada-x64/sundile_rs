use anyhow::*;

mod types;
mod util;
mod converters;
use converters::*;

pub(crate) mod internal {
    pub use crate::types::*;
    pub use crate::util::*;
}
use crate::internal::*;

pub mod prelude {
    pub use crate::types::*;
}

pub fn build() -> Result<()> {
    echo("Starting build...");
    // Check if build is necessary.

    // Load assets
    let mut asset_dir = std::env::current_dir()?;
    asset_dir.push("assets");
    echo(asset_dir.as_path().to_str().unwrap());

    let assets = AssetsData {
        shaders: shader::DataMap::load(&asset_dir, "shaders", "wgsl"),
        textures: load_generic_data(&asset_dir, "textures")?, //texture::load()?,
        models: model::DataMap::load(&asset_dir, "models", "obj"),
        audio: load_generic_data(&asset_dir, "audio")?, //audio::load()?,
        fonts: load_generic_data(&asset_dir, "fonts")?, //font::load()?,
    };

    use std::io::Write;
    let path = std::env::current_dir()?.join("data.bin"); //format!("{}/../../../data.bin", std::env::var("OUT_DIR")?); //This probably isn't ideal.
    std::fs::File::create(&path)?.write_all(&bincode::serialize(&assets)?)?;
    echo(format!("Wrote to {}", &path.to_str().unwrap()).as_str());

    Ok(())
}

pub fn parse(bin: &[u8], render_target: &sundile_graphics::render_target::RenderTarget) -> Assets {
    bincode::deserialize::<AssetsData>(bin).unwrap().parse(&render_target).unwrap()
}