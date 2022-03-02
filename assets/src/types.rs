use std::collections::HashMap;
use serde::*;
use std::path::*;
use graphics::render_target::RenderTarget;
use crate::converters::*;
use anyhow::*;

pub type AssetMap = HashMap<String, Vec<u8>>;

pub trait Datatype<T> {
    fn load(path: &PathBuf) -> Self;
    fn convert(self, render_target: &RenderTarget) -> Result<T>;
}
pub trait Map<EmbeddedMap> {
    fn load(asset_dir: &PathBuf) -> Self;
    fn convert(self, render_target: &RenderTarget) -> Result<EmbeddedMap>;
}

#[derive(Serialize, Deserialize)]
pub struct AssetsData {
    pub shaders: shader::Map,
    pub textures: AssetMap,
    pub models: model::DataMap,
    pub audio: AssetMap,
    pub fonts: AssetMap,
}

pub struct Assets {
    pub shaders: shader::Map,
    pub textures: AssetMap,
    pub models: model::EmbeddedMap,
    pub audio: AssetMap,
    pub fonts: AssetMap,
}

impl core::fmt::Debug for AssetsData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Assets [\nshaders: {:?}\ntextures: {:?}\nmodels: {:?}\naudio: {:?}\nfonts: {:?}]",
            self.shaders.keys(),
            self.textures.keys(),
            self.models.keys(),
            self.audio.keys(),
            self.fonts.keys(),
        )
    }
}

impl AssetsData {
    pub fn convert(self, render_target: &RenderTarget) -> Result<Assets> {
        Ok(Assets {
            shaders: self.shaders,
            textures: self.textures,
            models: self.models.convert(&render_target)?,
            audio: self.audio,
            fonts: self.fonts,
        })
    }
}