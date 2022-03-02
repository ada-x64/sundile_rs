use std::collections::HashMap;
use anyhow::*;
use crate::*;

pub type Map = HashMap<String, String>;

pub fn load(asset_dir: &PathBuf) -> Result<Map> {
    // TODO: Compile here so you don't run into validation errors at runtime.
    // Maybe use naga or one of the inline macro libs.
    //Iterate over textures in ASSET_DIR/shaders and compile them into SPIR-V.
    //Possibly compress this data into binary to be included in EXE / .dat file.
    
    echo("Loading shaders...");
    let mut res = Map::new();
    let mut path = asset_dir.to_owned();
    path.push("shaders");

    let dir = read_dir(path)?.into_iter();
    for entry in dir {
        let entry = entry?;
        let filename = entry.path().file_stem().unwrap().to_str().unwrap().to_string();
        let mut buffer = String::new();
        let mut file = File::open(entry.path())?;
        file.read_to_string(&mut buffer)?;
        res.insert(filename, buffer);
    }
    Ok(res)
}