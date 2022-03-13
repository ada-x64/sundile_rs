use std::collections::HashMap;
use std::path::*;

use sundile_graphics::prelude::HeadlessRenderTarget;
use sundile_graphics::render_target::RenderTarget;

// ---
// The below types deal with Assets as they are presented to the game.
// ---

//TODO: Create crate for proc_macro.
/// Asset trait required for types to be handled by [AssetMap] / [AssetTypeMap].
pub trait Asset {
    fn get_type_name(&self) -> &'static str;
}

/// Maps assets of a particular type. Typically used inside of [AssetTypeMap].
/// Because we want [AssetTypeMap] to be extensible, this type takes in a type that satisfies the Asset trait.
pub type AssetMap<'a> = HashMap<String, Box<dyn Asset + 'a>>;
/// Maps assets in the pattern `asset_map["asset_type"]["asset_name"]`, e.g. `asset_map["models"]["test_cube"]`
pub type AssetTypeMap<'a> = HashMap<String, AssetMap<'a>>;
pub trait AssetTypeMapFns {
    fn combine(&mut self, other: Self);
    fn asset<'f, S, T>(&'f self, asset_type_name: S, asset_name: S) -> T where S : Into<String>, T: From<&'f Box<dyn Asset + 'f>>;
}
impl<'a> AssetTypeMapFns for AssetTypeMap<'a> {
    fn combine(&mut self, other: AssetTypeMap<'a>) {
        for (asset_type, new_map) in other.into_iter() {
            match self.get_mut(&asset_type) {
                Some(old_map) => {
                    old_map.extend(new_map);
                }
                None => {
                    self.insert(asset_type, new_map);
                }
            }
        }
    }
    
    fn asset<'f, S, T>(&'f self, asset_type_name: S, asset_name: S) -> T where S : Into<String>, T: From<&'f Box<dyn Asset + 'f>> {
        self.get(&asset_type_name.into()).unwrap().get(&asset_name.into()).unwrap().into()
    }
}


// ---
// The below types deal with Assets as they are stored in bytecode / data.bin.
// ---

/// [AssetMap] to be used with bytecode data. Automatically derives serialization.
pub type BincodeAssetMap = HashMap<String, Vec<u8>>;
/// [AssetTypeMap] to be used with bytecode data. Automatically derives serialization.
pub type BincodeAssetTypeMap = HashMap<String, BincodeAssetMap>;


// ---
// The below types deal with Assets as they are loaded from disk in the raw form.
// ---

/// Represents an [Asset] as loaded directly from disk.
/// This is an intermediary form, which should be converted to Asset types and loaded into [AssetMap]s.
pub trait RawAsset<AssetType> where AssetType : Asset {
    /// Loads an individual asset from disk and stores it in this type.
    fn from_disk(path: &PathBuf) -> Self;
    /// Converts this type to the AssetType to be used with the engine.
    fn to_asset<'f>(self, render_target: &AssetBuilder<'f>) -> AssetType;
}

/// Type that converts a specified RawAsset type to a specified Asset type.
pub trait RawAssetMapper {
    /// Loads all relevant files from disk.
    /// Tip: call RawAsset::load_from_disk internally.
    fn load(&mut self, asset_dir: &PathBuf);
    /// Converts from raw data to the representation used in-game.
    fn to_asset_map<'a>(self: Box<Self>, asset_builder: &AssetBuilder) -> AssetMap<'a>;
    /// Deserializes from bytecode into raw asset data.
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap);
    /// Serializes self from raw asset data to bytecode
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap;
}

/// Hashmap from string to RawAsset.
pub type RawAssetMap<'a, AssetType> = HashMap<String, Box<dyn RawAsset<AssetType> + 'a>>;

/// Builds assets from a RenderTarget.
/// Only intended to last for the duration of a deserialization call.
pub struct AssetBuilder<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}
impl<'a> From<&'a HeadlessRenderTarget> for AssetBuilder<'a> {
    fn from(other: &'a HeadlessRenderTarget) -> Self {
        AssetBuilder {
            device: &other.device,
            queue: &other.queue
        }
    }
}
impl<'a> From<&'a RenderTarget> for AssetBuilder<'a> {
    fn from(other: &'a RenderTarget) -> Self {
        AssetBuilder {
            device: &other.device,
            queue: &other.queue
        }
    }
}