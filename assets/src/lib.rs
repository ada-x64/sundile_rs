use anyhow::*;
use graphics::prelude::RenderTarget;
use std::fs::*;
use std::io::Read;
use std::path::*;

mod types;
use crate::types::*;
mod util;
use crate::util::*;

mod converters;
use converters::*;

pub mod prelude {
    pub use crate::types::*;

    pub type ShaderMap = crate::shader::Map;
    pub type ModelMap = crate::model::EmbeddedMap;
    //etc
}

pub(crate) mod internal {
    pub use crate::types::*;
    pub use crate::util::*;
}


pub fn build() -> Result<()> {
    echo("Starting build...");
    // Check if build is necessary.

    // Load assets
    let mut asset_dir = std::env::current_dir()?;
    asset_dir.push("assets");
    echo(asset_dir.as_path().to_str().unwrap());

    let assets = AssetsData {
        shaders: shader::load(&asset_dir)?,
        textures: load_generic_data(&asset_dir, "textures")?, //texture::load()?,
        models: model::DataMap::load(&asset_dir),
        audio: load_generic_data(&asset_dir, "audio")?, //audio::load()?,
        fonts: load_generic_data(&asset_dir, "fonts")?, //font::load()?,
    };

    use std::io::Write;
    let path = format!("{}/../../../data.bin", std::env::var("OUT_DIR")?); //This probably isn't ideal.
    echo(&path.as_str());
    std::fs::File::create(path)?.write_all(&bincode::serialize(&assets)?)?;

    Ok(())
}

pub fn load_from_bin(render_target: &RenderTarget) -> Result<Assets> {
    let path = std::env::current_exe()?.parent().unwrap().join("data.bin");
    echo(path.to_str().unwrap());
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;
    Ok(bincode::deserialize::<AssetsData>(&buffer[..])?.convert(render_target)?)
}