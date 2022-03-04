use std::collections::HashMap;
use serde::*;
use std::path::*;
use sundile_graphics::render_target::RenderTarget;
use crate::converters::*;
use crate::util::*;
use anyhow::*;

pub type AssetMap = HashMap<String, Vec<u8>>;

pub trait DataType<T> {
    fn load(path: &PathBuf) -> Self;
    fn convert(self, render_target: &RenderTarget) -> Result<T>;
}

pub trait Map<EmbeddedType> {
    fn load<'a>(asset_dir: &PathBuf, subdir: &'a str, extension: &'a str) -> Self;
    fn convert(self, render_target: &RenderTarget) -> Result<HashMap<String, EmbeddedType>>;
}

fn search_directory<'a>(path: &PathBuf, extension: &'a str) -> Result<Vec<PathBuf>> {
    let mut res = Vec::new();
    for item in path.read_dir()? {
        let path = item?.path();
        if path.is_dir() {
            res.append(&mut search_directory(&path, extension)?);
        }
        else if path.extension().unwrap_or(&std::ffi::OsStr::new("")) == extension {
            res.push(path);
        }
    }
    Ok(res)
}

impl<T, EmbeddedType> Map<EmbeddedType> for HashMap<String, T> where T: DataType<EmbeddedType> {
    fn load<'a>(asset_dir: &PathBuf, subdir: &'a str, extension: &'a str) -> Self {
        echo(format!("Loading {}/*.{} ", subdir, extension).as_str());
        let mut res = HashMap::new();
        let mut path = asset_dir.to_owned();
        path.push(subdir);
        for path in search_directory(&path, extension)
            .expect(format!("Failed to traverse {}", path.display()).as_str())
            {
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            echo(path.file_name().unwrap().to_str().unwrap());
            res.insert(name, T::load(&path));
        }
        res
    }

    fn convert(self, render_target: &RenderTarget) -> Result<HashMap<String, EmbeddedType>> {
        Ok(HashMap::from_iter(
            self.into_iter().map(
                |(name, data)| -> (String, EmbeddedType) {
                    let data = data.convert(render_target)
                        .expect(format!("Unable to convert {}", &name).as_str());
                    (name, data)
                }
            )
        ))
    }
}

//TODO: Make this a hashmap so it's extensible by libraries.
#[derive(Serialize, Deserialize)]
pub struct AssetsData {
    pub shaders: shader::DataMap,
    pub textures: AssetMap,
    pub models: model::DataMap,
    pub audio: AssetMap,
    pub fonts: AssetMap,
}

//TODO: add impl to check if asset exists, possibly load default value if none found
pub struct Assets {
    pub shaders: shader::EmbeddedMap,
    pub textures: AssetMap,
    pub models: model::EmbeddedMap,
    pub audio: AssetMap,
    pub fonts: AssetMap,
}

// impl core::fmt::Debug for AssetsData {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//         write!(f, "Assets [\nshaders: {:?}\ntextures: {:?}\nmodels: {:?}\naudio: {:?}\nfonts: {:?}]",
//             self.shaders.keys(),
//             self.textures.keys(),
//             self.models.keys(),
//             self.audio.keys(),
//             self.fonts.keys(),
//         )
//     }
// }

impl AssetsData {
    pub fn parse(self, render_target: &RenderTarget) -> Result<Assets> {
        Ok(Assets {
            shaders: self.shaders.convert(&render_target)?,
            textures: self.textures,
            models: self.models.convert(&render_target)?,
            audio: self.audio,
            fonts: self.fonts,
        })
    }
}