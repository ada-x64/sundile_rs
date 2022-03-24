use std::collections::HashMap;
use std::path::*;
use std::fs::*;
use std::io::Read;
use crate::prelude::*;
use sundile_graphics::*;

pub type TextureData = Vec<u8>;

impl RawAsset<Texture> for TextureData {
    /// Loads in the texture file as raw bytes.
    fn from_disk(path: &PathBuf) -> Self {
        let mut buffer = Vec::<u8>::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    /// Creates a [Texture] from the serialized bytes.
    fn to_asset(self, asset_builder: &AssetBuildTarget) -> Texture {
        Texture::from_bytes(asset_builder.device, asset_builder.queue, &self[..], "statically loaded texture", false).expect("Unable to create texture!")
    }
}

///TODO: Figure out a way to press this into a [sundile_graphics::TextureAtlas].
pub struct Mapper {
    map: HashMap<String, TextureData>
}
impl Mapper {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }
}
impl RawAssetMapper for Mapper {
    fn load(&mut self, asset_dir: &PathBuf) {
        crate::util::generic_load::<TextureData, Texture>(&mut self.map, asset_dir, "textures", "png");
    }
    fn to_asset_map<'a>(self: Box<Self>, builder: &AssetBuildTarget) -> AssetMap {
        //TODO: Compress this into a TextureAtlas
        crate::util::generic_to_asset_map::<TextureData, Texture>(self.map, builder)
    }
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap) {
        crate::util::generic_load_bin_map::<TextureData, Texture>(&mut self.map, bin_map);
    }
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap {
        crate::util::generic_to_bin_map::<TextureData, Texture>(self.map)
    }
}