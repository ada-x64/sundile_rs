use std::fs::*;
use std::path::*;
use crate::types::*;
use anyhow::*;
use std::io::Read;

#[allow(dead_code)]
pub fn dump() {
    echo("Dumped!");
    echo("Args:");
    for arg in std::env::args() {
        echo(arg.as_str());
    }
    echo("Vars:");
    for (var, val) in std::env::vars() {
        echo(format!("{} = {}", var, val).as_str());
    }
    panic!();
}

#[allow(dead_code)]
pub fn echo<'a>(msg: &'a str) {
    use std::io::Write;
    std::io::stdout().write_all(format!("{}{}",msg,'\n').as_bytes()).expect("fakjsflsd");
}

#[allow(dead_code)]
pub fn load_generic_data(asset_dir: &PathBuf, subdir: &'static str) -> Result<AssetMap> {
    echo(format!("load_generic_data for {}", &subdir).as_str());
    let mut res = AssetMap::new();

    let mut path = asset_dir.to_owned();
    path.push(subdir);

    let dir = read_dir(path)?.into_iter();
    for entry in dir {
        let entry = entry?;
        let filename = entry.path().file_stem().unwrap().to_str().unwrap().to_string();
        let mut buffer = Vec::<u8>::new();
        let mut file = File::open(entry.path())?;
        file.read_to_end(&mut buffer)?;
        res.insert(filename, buffer);
    }
    Ok(res)
}