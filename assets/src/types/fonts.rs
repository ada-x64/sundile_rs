use std::collections::HashMap;
use std::path::*;
use std::fs::*;
use std::io::Read;
use crate::prelude::*;
use sundile_graphics::*;

impl RawAsset<Font> for Font {
    /// Loads in the font file as raw bytes.
    fn from_disk(path: &PathBuf) -> Self {
        let mut buffer = Vec::<u8>::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        Self { data: buffer}
    }

    /// Simply returns the byte vector.
    fn to_asset(self, _: &AssetBuildTarget) -> Font {
        self
    }
}

pub struct Mapper {
    map: HashMap<String, Font>
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
        crate::util::generic_load::<Font, Font>(&mut self.map, asset_dir, "fonts", "ttf");
    }
    fn to_asset_map<'a>(self: Box<Self>, builder: &AssetBuildTarget) -> AssetMap {
        crate::util::generic_to_asset_map::<Font, Font>(self.map, builder)
    }
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap) {
        crate::util::generic_load_bin_map::<Font, Font>(&mut self.map, bin_map);
    }
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap {
        crate::util::generic_to_bin_map::<Font, Font>(self.map)
    }
}