
use sundile_common::*;
use std::collections::HashMap;
use std::path::*;
use crate::*;
use log::info;

/// Loads asset data into a binary. Intended to be used in build scripts to statically load assets.
pub struct Serializer<'a> {
    mappers: HashMap<String, Box<dyn RawAssetMapper + 'a>>,
    out_path: Option<PathBuf>,
    asset_directory: Option<PathBuf>,
}
impl<'a> Serializer<'a> {
    /// Creates a new serializer with default options.
    pub fn new() -> Self {
        Self {
            mappers: HashMap::new(),
            out_path: None,
            asset_directory: None,
        }
    }
    /// Adds an asset map to be serialized.
    pub fn with_mapper<S>(mut self, asset_type_name: S, mapper: impl RawAssetMapper + 'a) -> Self where S: Into<String> {
        self.mappers.insert(asset_type_name.into(), Box::new(mapper));
        self
    }
    /// Sets the output directory. Data will be serialized to out_dir/data.bin.
    /// The default out_path is "./".
    pub fn with_out_path<P>(mut self, path: P) -> Self where P : Into<PathBuf> {
        self.out_path = Some(path.into());
        self
    }
    /// Sets the asset directory. The asset compiler should then load data from asset_directory/(asset_type)/*.type_extension
    /// The default in path is "./assets/"
    pub fn with_asset_directory<P>(mut self, path: P) -> Self where P : Into<PathBuf> {
        self.asset_directory = Some(path.into());
        self
    }
    /// Iterates over the given compilers, loads and serializes the data, outputs that data to out_path/data.bin, and returns the binary.
    // TODO: Should this function be responsible for caching or should we shunt that to the individual asset compilers?
    pub fn serialize(self) -> Vec<u8> {
        info!("Serializing assets...");

        let out_path = self.out_path
            .unwrap_or("./".into())
            .join("data.bin");

        let in_path = self.asset_directory
            .unwrap_or("./assets/".into());

        let mut out_map = BincodeAssetTypeMap::new();
        for (name, mut mapper) in self.mappers {
            mapper.load(&in_path);
            let bin_map = mapper.to_bin_map();
            out_map.insert(name.to_owned(), bin_map);
        }
        let bin = bincode::serialize(&out_map).expect("Unable to serialize RawAssetTypeMap");

        use std::io::Write;
        std::fs::File::create(out_path).expect("Unable to create file at out_path")
            .write(&bin[..]).expect("Unable to write to bin");

        info!("...Done!");
        bin
    }
}

impl<'a> Default for Serializer<'a> {
    fn default() -> Self {
        Self::new()
            .with_mapper("shaders", types::shaders::Mapper::new())
            .with_mapper("models", types::models::Mapper::new())
            .with_mapper("textures", types::textures::Mapper::new())
            .with_mapper("fonts", types::fonts::Mapper::new())
            //etc
    }
}

/// Struct that deserializes static data as serialized by this library.
pub struct Deserializer<'a> {
    mappers: HashMap<String, Box<dyn RawAssetMapper + 'a>>,
    panic: bool,
}
impl<'a> Deserializer<'a> {
    /// Creates a new deserializer with default rules.
    pub fn new() -> Self {
        Self {
            mappers: HashMap::new(),
            panic: true,
        }
    }
    /// Adds an asset map to be deserialized.
    pub fn with_mapper<S>(mut self, asset_type_name: S, mapper: impl RawAssetMapper + 'a) -> Self where S: Into<String> {
        self.mappers.insert(asset_type_name.into(), Box::new(mapper));
        self
    }
    /// Determines if [Deserializer::deserialize] will panic if it cannot convert all available data.
    pub fn with_panic(mut self, enabled: bool) -> Self {
        self.panic = enabled;
        self
    }
    /// Parses the bin. May panic if it cannot parse the binary into an AssetTypeMap or if no mapper exists for an asset type within that binary.
    pub fn deserialize<'f, BuilderType>(self, bin: &[u8], asset_builder: &'f BuilderType) -> AssetTypeMap
    where &'f BuilderType: Into<AssetBuildTarget<'f>> {
        info!("Deserializing assets...");
        let builder = asset_builder.into();
        let mut map_in = bincode::deserialize::<BincodeAssetTypeMap>(bin).expect("Unable to read bin!");
        let mut map_out = AssetTypeMap::new();

        for (name, mut mapper) in self.mappers {
            let bin_map = match map_in.remove(&name) {
                Some(bin_map) => bin_map,
                None => continue
            };
            mapper.load_bin_map(bin_map);
            map_out.insert_asset_map(mapper.to_asset_map(&builder));
        }

        if !map_in.is_empty() && self.panic {
            panic!("Binary not fully read.")
        }

        info!("...Done!");
        map_out
    }
}

impl<'a> Default for Deserializer<'a> {
    fn default() -> Self {
        Self::new()
            .with_mapper("shaders", types::shaders::Mapper::new())
            .with_mapper("models", types::models::Mapper::new())
            .with_mapper("textures", types::textures::Mapper::new())
            .with_mapper("fonts", types::fonts::Mapper::new())
            //etc
    }
}