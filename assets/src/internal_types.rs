use std::any::*;
use std::collections::HashMap;
use std::path::*;
use std::rc::Rc;
use anyhow::*;
use std::result::Result::{Ok, Err};
use thiserror::Error;

use sundile_graphics::prelude::HeadlessRenderTarget;
use sundile_graphics::render_target::RenderTarget;

// ---
// The below types deal with Assets as they are presented to the game.
// ---

/// Error enum for Asset-related functionality.
#[derive(Error, Debug, PartialEq)]
pub enum AssetError {
    #[error("Could not match asset types.")]
    InvalidType,
    #[error("Could not convert to owned type. This probably means there are existing references to this value!")]
    InvalidTake,
    #[error("No asset map with type name '{0}'")]
    AssetTypeNotFound(String),
    #[error("No asset found with name '{0}")]
    AssetNotFound(String),
}

/// Asset storage type required for [AssetMap] / [AssetTypeMap].
#[derive(Debug)]
pub struct AssetStorage {
    // TODO: Convert to Arc<Mutex<dyn Any>> and reimplement, cascading upwards.
    value: Rc<dyn Any>
}
impl std::ops::Deref for AssetStorage {
    type Target = Rc<dyn Any>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl AssetStorage {
    /// Constructs a new Asset
    pub fn new<T>(value: T) -> Self where T: 'static {
        Self {
            value: Rc::new(value),
        }
    }
    /// Tries to return a reference counted pointer to the underlying asset.
    /// This function fails if the asset is not of type T.
    pub fn try_get<T>(&self) -> Result<Rc<T>> where T: 'static {
        match self.value.clone().downcast::<T>() {
            Ok(val) => Ok(val),
            Err(_) => Err(anyhow!(AssetError::InvalidType))
        }
    }
    /// Tries to convert to a specified type, returning an owned value and consuming self in the process.
    /// If this function fails, the error value will contain self.
    /// This function removes assets from the heap and will fail if there are multiple extant references to that asset.
    pub fn try_take<T>(self) -> Result<T, (AssetError, Self)> where T: 'static {
        match self.value.downcast::<T>() {
            Ok(cast_value) =>  match Rc::try_unwrap(cast_value) {
                Ok(inner) => Ok(inner),
                Err(value) => Err((AssetError::InvalidTake, Self { value })),
            }
            Err(value) => Err((AssetError::InvalidType, Self { value } )),
        }
    }
}

/// Maps assets of a particular type. Typically used inside of [AssetTypeMap].
/// Because we want [AssetTypeMap] to be extensible, this type takes in a type that satisfies the Asset trait.
#[derive(Debug)]
pub struct AssetMap {
    pub type_id: TypeId,
    map: HashMap<String, AssetStorage>,
}
impl AssetMap {
    /// Creates a new AssetMap from the specified type.
    pub fn new<T>() -> Self where T: 'static {
        Self {
            type_id: TypeId::of::<T>(),
            map: HashMap::new(),
        }
    }
    /// Creates a new AssetMap from the specified TypeId
    pub fn from_type_id(type_id: TypeId) -> Self {
        Self {
            type_id,
            map: HashMap::new(),
        }
    }
    /// Creates a new AssetMap, automatically inserting the asset.
    pub fn from_asset<S, T>(name: S, asset: T) -> Self where S: Into<String>, T: 'static {
        Self {
            type_id: TypeId::of::<T>(),
            map: HashMap::from_iter([(name.into(), AssetStorage::new(asset))]),
        }
    }
    /// Creates a new AssetMap from a HashMap.
    pub fn from_map<S, T>(map: HashMap<S, T>) -> Self where S:Into<String>, T: 'static {
        let map = HashMap::from_iter(
            map.into_iter()
            .map(|(name, asset)| -> (String, AssetStorage) {
                (name.into(), AssetStorage::new(asset))
            })
        );

        Self {
            type_id: TypeId::of::<T>(),
            map
        }
    } 

    /// Tries to extend this map with another map. Fails if types do not match.
    pub fn try_extend(&mut self, other: AssetMap) -> Result<&Self> {
        if other.type_id == self.type_id {
            self.map.extend(other.map);
            return Ok(self);
        }
        return Err(anyhow!(AssetError::InvalidType))
    }
    /// Tries to insert an asset. Will return an error if the value is not of the appropriate type.
    /// Like HashMap::insert, this function will return a Some(T) if there is a value at that place.
    /// This function fails if an asset exists at the specified position and it cannot be converted to T.
    pub fn try_insert<S, T>(&mut self, name: S, asset: T) -> Result<Option<T>, (AssetError, Option<AssetStorage>)> where S: Into<String>, T: 'static {
        if TypeId::of::<T>() == self.type_id {
            match self.map.insert(name.into(), AssetStorage::new(asset)) {
                Some(asset_storage) => {
                    match asset_storage.try_take() {
                        Ok(val) => return Ok(Some(val)),
                        Err((err, storage)) => return Err((err, Some(storage))),
                    }
                },
                None => return Ok(None),
            }
        }
        return Err((AssetError::InvalidType, None));
    }
    /// Tries to get a reference to an asset.
    /// This function fails if it cannot find the asset with the given name, or if it cannot convert the asset to the specified type.
    pub fn try_get<S, T>(&self, name: S) -> Result<Rc<T>> where S: Into<String>, T: 'static {
        let name = name.into();
        match self.map.get(&name) {
            Some(storage) => {
                storage.try_get()
            }
            None => Err(anyhow!(AssetError::AssetNotFound(name)))
        }
    }
    /// Tries to take an asset. If the asset is found, it is removed from the map.
    /// This function fails if it cannot find the asset with the given name, or if it cannot convert the asset to the specified type.
    /// This function removes assets from the heap and will fail if there are multiple extant references to that asset.
    pub fn try_take<S, T>(&mut self, name: S) -> Result<T> where S: Into<String>, T: 'static {
        let name = name.into();
        match self.map.remove(&name) {
            Some(val) => {
                match val.try_take() {
                    Ok(val) => Ok(val),
                    Err((err, storage)) => {
                        self.map.insert(name, storage);
                        Err(anyhow!(err))
                    }
                }
            }
            None => Err(anyhow!(AssetError::AssetNotFound(name)))
        }
    }
    /// Tries to convert to the specified type, consuming itself in the process.
    /// This function removes assets from the heap and will fail if there are multiple extant references to that asset.
    /// On failure, this function returns both the error type and the original [AssetMap].
    pub fn try_into<T>(self) -> Result<HashMap<String, T>, (AssetError, Self)> where T: 'static {
        let mut res = HashMap::new();
        let mut errdata = None;
        for (name, storage) in self.map {
            match storage.try_take::<T>() {
                Ok(asset) => {
                    res.insert(name, asset);
                }
                Err((err, storage)) => {
                    errdata = Some((
                        err,
                        Self {
                            type_id: storage.type_id(),
                            map: HashMap::from_iter([(name, storage)]),
                        },
                    ));
                    break;
                }
            }
        }
        if errdata.is_some() {
            let (err, mut asset_map) = errdata.unwrap();
            for (name, asset) in res {
                asset_map.map.insert(name, AssetStorage::new(asset));
            }
            return Err((err, asset_map));
        }
        Ok(res)
    }
    /// Tries to convert to a hashmap containing references to values of the specified type.
    pub fn try_as<T>(&self) -> Result<HashMap<String, Rc<T>>> where T: 'static {
        let mut res = HashMap::new();
        for (name, storage) in &self.map {
            match storage.try_get::<T>() {
                Ok(asset) => {
                    res.insert(name.to_owned(), asset);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(res)
    }
}

/// Typemap for assets. Thin wrapper over HashMap<String, [AssetMap]>.
#[derive(Debug)]
pub struct AssetTypeMap {
    map: HashMap<String, AssetMap>
}
impl AssetTypeMap {
    /// Creates an empty [AssetTypeMap].
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    
    /// Combines this asset typemap with another in-place.
    pub fn try_combine(&mut self, other: AssetTypeMap) -> Result<&Self> {
        for (asset_type, new_map) in other.map.into_iter() {
            match self.map.get_mut(&asset_type) {
                Some(old_map) => {
                    match old_map.try_extend(new_map) {
                        Ok(_) => {},
                        Err(e) => return Err(e),
                    }
                }
                None => {
                    self.map.insert(asset_type, new_map);
                }
            }
        }
        Ok(self)
    }

    /// Tries to insert an asset into the specified typemap, creating the typemap if it does not exist.
    /// Following HashMap::insert, this function will return Some(T) if there was previously an asset in the specified slot, and None otherwise.
    /// This function fails if the specified asset_map exists, but its type does not match the asset passed in.
    /// This function fails if an asset exists at the specified position and it cannot be converted to T.
    pub fn try_insert_asset<S, T>(&mut self, ty: S, name: S, asset: T) -> Result<Option<T>, (AssetError, Option<AssetStorage>)> where S: Into<String>, T: 'static {
        let ty = ty.into();
        match self.map.get_mut(&ty) {
            Some(map) => {
                map.try_insert(name, asset)
            }
            None => {
                self.map.insert(ty, AssetMap::from_asset(name, asset));
                Ok(None)
            }
        }
    }
    
    /// Tries to get an asset reference. Will return an error if it cannot find the type or asset name, and if it cannot be converted to the specified type.
    pub fn try_get_asset<S, T>(&self, ty: S, name: S) -> Result<Rc<T>> where S: Into<String>, T: 'static {
        let ty = ty.into();
        match self.map.get(&ty) {
            Some(map) => {
                map.try_get(name)
            }
            None => Err(anyhow!(AssetError::AssetTypeNotFound(ty)))
        }
    }
    
    /// Tries to take an asset as an owned value, removing it from the map in the process.
    /// This function removes assets from the heap and will fail if there are multiple extant references to that asset.
    pub fn try_take_asset<S, T>(&mut self, ty: S, name: S) -> Result<T> where S: Into<String>, T: 'static {
        let ty = ty.into();
        match self.map.get_mut(&ty) {
            Some(val) => {
                val.try_take(name)
            }
            None => Err(anyhow!(AssetError::AssetTypeNotFound(ty)))
        }
    }

    /// Directly inserts an [AssetMap].
    pub fn insert_asset_map<S>(&mut self, ty: S, map: AssetMap) -> Option<AssetMap> where S: Into<String> {
        self.map.insert(ty.into(), map)
    }
    
    /// Takes a HashMap of specific assets and inserts them into the table, converting them into the [Asset] storage type.
    pub fn insert_map<S,T>(&mut self, ty: S, map: HashMap<S, T>) -> Option<AssetMap> where S: Into<String>, T: 'static {
        let mut asset_map = AssetMap::new::<T>();
        for (name, asset) in map {
            asset_map.try_insert(name, asset).expect("If you're seeing this, something went wrong.");
        }
        self.map.insert(ty.into(), asset_map)
    }
    
    /// Tries to return a HashMap with type-converted asset references. Will return an error if it cannot find a map for the type, or if any member cannot be converted.
    /// This function removes assets from the heap and will fail if there are multiple extant references to that asset.
    pub fn try_get_asset_map<S,T>(&self, ty: S) -> Result<HashMap<String, Rc<T>>> where S: Into<String>, T: 'static {
        let ty = ty.into();
        match self.map.get(&ty) {
            Some(map) => {
                map.try_as()
            }
            None => Err(anyhow!(AssetError::AssetTypeNotFound(ty)))
        }
    }
    
    /// Returns an owned, type-converted HashMap by removing it from the AssetTypeMap.
    /// This function fails if it cannot find a map for the type, or if any member cannot be converted.
    /// On failure, the AssetMap is not removed.
    /// This function removes assets from the heap and will fail if there are multiple extant references to that asset.
    pub fn try_take_asset_map<S,T>(&mut self, ty: S) -> Result<HashMap<String, T>> where S: Into<String>, T: 'static {
        let ty = ty.into();
        match self.map.remove(&ty) {
            Some(map) => {
                match map.try_into() {
                    Ok(val) => Ok(val),
                    Err((err, map)) => {
                        self.map.insert(ty, map);
                        Err(anyhow!(err))
                    }
                }
            }
            None => Err(anyhow!(AssetError::AssetTypeNotFound(ty)))
        }
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
pub trait RawAsset<AssetType> where AssetType : Any {
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
    fn to_asset_map<'a>(self: Box<Self>, asset_builder: &AssetBuilder) -> AssetMap;
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