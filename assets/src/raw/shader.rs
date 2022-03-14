use std::collections::HashMap;
use wgpu::*;
use std::path::*;
use std::fs::*;
use std::io::Read;
use naga::{
    valid::{ValidationFlags, Validator, Capabilities},
    front::wgsl,
    back::spv,
    back::spv::WriterFlags,
};

use crate::prelude::*;

pub type ShaderData = Vec<u32>;

impl RawAsset<ShaderModule> for ShaderData {
    /// Loads in the shader file, asserts it is valid, and converts it to binary SPIR-V.
    fn from_disk(path: &PathBuf) -> Self {
        //Note: Not just loading this in because all the frontends expect words but reading a file gives bytes.
        let mut naga = std::process::Command::new("naga").args([path.to_str().unwrap()]).spawn().expect("Unable to spawn naga. Is it installed?");
        if !naga.wait().unwrap().success() {
            panic!()
        }
    
        let mut buffer = String::new();
        let mut file = File::open(path).unwrap();
        file.read_to_string(&mut buffer).unwrap();
        let source_text = buffer.as_str();
        let module = wgsl::parse_str(source_text).unwrap();
        let info = Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module).unwrap();
        let options = spv::Options {
            flags: WriterFlags::LABEL_VARYINGS | WriterFlags::CLAMP_FRAG_DEPTH | WriterFlags::DEBUG,
            ..Default::default()
        };
        spv::write_vec(&module, &info, &options, None).unwrap()
    }

    /// Converts the SPIR-V binary to a shader module.
    fn to_asset(self, asset_builder: &AssetBuilder) -> ShaderModule {
        unsafe {
            asset_builder.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: self.into(),
            })
        }
    }
}

pub type ShaderMapper = HashMap<String, ShaderData>;

impl RawAssetMapper for ShaderMapper {
    fn load(&mut self, asset_dir: &PathBuf) {
        crate::util::generic_load(self, asset_dir, "shaders", "wgsl");
    }
    fn to_asset_map<'a>(self: Box<Self>, builder: &AssetBuilder) -> AssetMap {
        crate::util::generic_to_asset_map(*self, builder)
    }
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap) {
        crate::util::generic_load_bin_map(self, bin_map);
    }
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap {
        crate::util::generic_to_bin_map(*self)
    }
}