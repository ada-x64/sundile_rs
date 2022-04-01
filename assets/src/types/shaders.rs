use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use sundile_graphics::prelude::wgpu;
use std::path::*;
use std::fs::*;
use std::io::Read;

use crate::*;

#[derive(Serialize, Deserialize)]
pub struct ShaderData {
    data: Vec<u8>,
}

impl RawAsset<wgpu::ShaderModule> for ShaderData {
    /// Loads in the shader file, asserts it is valid, and converts it to binary SPIR-V.
    fn from_disk(path: &PathBuf) -> Self {
        let in_str = path.to_str().unwrap();
        let out_str = path.file_name().unwrap().to_string_lossy() + "spv";

        std::process::Command::new("naga")
            .args([in_str])
            .args([&*out_str])
            .spawn()
            .expect("Unable to spawn naga. Is it installed?")
            .wait()
            .unwrap();

        let mut buf = Vec::new();
        let mut file = File::open(&*out_str).unwrap();
        file.read_to_end(&mut buf).unwrap();
        Self {
            data: buf
        }

        // naga::back is not implemented for wasm32. To work around this, call the CLI.
        // This works because the Serialize functions should be called primarily from the build script that is run on a local machine before it is packed for the web.
        // #[cfg(target_arch="wasm32")]
        // {

        // } 
        
        // #[cfg(not(target_arch="wasm32"))]
        // {
        //     let mut buffer = String::new();
        //     let mut file = File::open(path).unwrap();
        //     file.read_to_string(&mut buffer).unwrap();
        //     let source_text = buffer.as_str();
        //     let module = wgsl::parse_str(source_text).unwrap();
        //     let info = Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module).unwrap();
        //     let options = spv::Options {
        //         flags: WriterFlags::LABEL_VARYINGS | WriterFlags::CLAMP_FRAG_DEPTH | WriterFlags::DEBUG,
        //         ..Default::default()
        //     };
        //     spv::write_vec(&module, &info, &options, None).unwrap()
        // }
    }

    /// Converts the SPIR-V binary to a shader module.
    fn to_asset(self, asset_builder: &AssetBuildTarget) -> wgpu::ShaderModule {
        unsafe {
            asset_builder.device.create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                label: None,
                source: wgpu::make_spirv_raw(&self.data[..]),
            })
        }
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