use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use sundile_common::*;
use std::path::*;
use std::fs::*;
use std::io::Read;

use crate::*;

#[derive(Serialize, Deserialize)]
pub struct ShaderData {
    data: String,
}

impl RawAsset<wgpu::ShaderModule> for ShaderData {
    /// Loads in the shader file, asserts it is valid, and converts it to binary SPIR-V.
    fn from_disk(path: &PathBuf) -> Self {
        match std::process::Command::new("naga")
            .arg(path.as_os_str())
            .spawn() {
                Ok(mut c) => {c.wait().ok();},
                Err(e) => {
                    use log::warn;
                    warn!("Could not validate shader using naga-cli.\n{}", e);
                    println!("cargo:warn=Could not validate shader using naga-cli.\n{}", e);
                }
            }

        let mut buffer = String::new();
        let mut file = File::open(path).unwrap();
        file.read_to_string(&mut buffer).unwrap();
        Self {
            data: buffer
        }
    }

    /// Converts the SPIR-V binary to a shader module.
    fn to_asset(self, asset_builder: &AssetBuildTarget) -> wgpu::ShaderModule {
        asset_builder.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.data.into()),
        })
    }
}

pub type Mapper = HashMap<String, ShaderData>;

impl RawAssetMapper for Mapper {
    fn load(&mut self, asset_dir: &PathBuf) {
        crate::util::generic_load(self, asset_dir, "shaders", "wgsl");
    }
    fn to_asset_map(self: Box<Self>, builder: &AssetBuildTarget) -> AssetMap {
        crate::util::generic_to_asset_map(*self, builder)
    }
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap) {
        crate::util::generic_load_bin_map(self, bin_map);
    }
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap {
        crate::util::generic_to_bin_map(*self)
    }
}