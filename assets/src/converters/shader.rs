use std::collections::HashMap;
use anyhow::*;
use crate::internal::*;
use wgpu::*;
use sundile_graphics::render_target::RenderTarget;
use std::path::*;
use std::fs::*;
use std::io::Read;
use naga::{
    valid::{ValidationFlags, Validator, Capabilities},
    front::wgsl,
    back::spv,
    back::spv::WriterFlags,
};

type ShaderData = Vec<u32>;

impl DataType<ShaderModule> for ShaderData {
    fn load(path: &PathBuf) -> Self {

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

    fn convert(self, render_target: &RenderTarget) -> Result<ShaderModule>  {
        Ok(unsafe {
            render_target.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
            label: None,
            source: self.into(),
            })
        })
    }
}

pub type DataMap = HashMap<String, ShaderData>;
pub type EmbeddedMap = HashMap<String, ShaderModule>;