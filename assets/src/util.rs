use sundile_common::*;
use std::collections::HashMap;
use anyhow::*;
use serde::*;
use serde::de::DeserializeOwned;
use std::path::*;
use std::any::Any;

use crate::internal_types::*;

/// Recursively searches the directory for any files with the matching extension.
/// Returns a Result containing a vector with every found path.
pub fn find_ext_recursive<'a>(path: &PathBuf, extension: &'a str) -> Result<Vec<PathBuf>> {
    let mut res = Vec::new();
    for item in path.read_dir()? {
        let path = item?.path();
        if path.is_dir() {
            res.append(&mut find_ext_recursive(&path, extension)?);
        }
        else if path.extension().unwrap_or(&std::ffi::OsStr::new("")) == extension {
            res.push(path);
        }
    }
    Ok(res)
}

/// Generically implements [RawAssetMapper::load]
pub fn generic_load<'a, RawAssetType, AssetType>
(mapper: &mut HashMap<String, RawAssetType>, asset_dir: &PathBuf, subdir: &'a str, ext: &'a str)
where RawAssetType: RawAsset<AssetType>, AssetType: Any {
    let mut path = asset_dir.to_owned();
    path.push(subdir);
    for path in crate::util::find_ext_recursive(&path, ext)
        .expect(format!("Failed to traverse {}", path.display()).as_str())
        {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        mapper.insert(name, RawAssetType::from_disk(&path));
    }
}

/// Generically implements [RawAssetMapper::to_asset_map]
pub fn generic_to_asset_map<'a, 'f, RawAssetType, AssetType>
(mapper: HashMap<String, RawAssetType>, builder: &AssetBuildTarget<'f>, ) -> AssetMap
where RawAssetType: RawAsset<AssetType>, AssetType: Any {
    AssetMap::from_map(
        HashMap::<String, AssetType>::from_iter(
            mapper.into_iter().map(
                |(name, data)| -> (String, AssetType) {
                    (name, data.to_asset(&builder))
                }
            )
        )
    )
}

/// Generically implements [RawAssetMapper::load_bin_map]
pub fn generic_load_bin_map<'a, RawAssetType, AssetType>
(mapper: &mut HashMap<String, RawAssetType>, bin_map: BincodeAssetMap)
where RawAssetType: RawAsset<AssetType> + DeserializeOwned, AssetType: Any{
    mapper.clear();
    for (name, vec) in bin_map {
        let data = bincode::deserialize(&vec[..]).expect("Unable to deserialize!");
        mapper.insert(name, data);
    }
}

/// Generically implements [RawAssetMapper::to_bin_map]
pub fn generic_to_bin_map<'a, RawAssetType, AssetType>
(mapper: HashMap<String, RawAssetType>) -> BincodeAssetMap
where RawAssetType: RawAsset<AssetType> + Serialize, AssetType: Any {
    let mut out = BincodeAssetMap::new();
    for (name, data) in mapper {
        out.insert(name.to_owned(), bincode::serialize(&data).expect("Unable to serialize!"));
    }
    out
}