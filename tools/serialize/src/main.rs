use std::process::exit;

use clap::Parser;
use sundile_assets::types;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Path to your asset directory.
    #[arg(short, long, default_value = "./assets")]
    in_path: String,
    /// Path to the output file.
    #[arg(short, long, default_value = "./")]
    out_path: String,

    /// Compile shaders
    #[arg(short, long)]
    shaders: bool,
    /// Compile models
    #[arg(short, long)]
    models: bool,
    /// Compile textures
    #[arg(short, long)]
    textures: bool,
    /// Compile fonts
    #[arg(short, long)]
    fonts: bool,
    /// Compile all compatible types.
    #[arg(short, long)]
    all: bool,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    if !args.all && !(args.shaders || args.models || args.textures || args.fonts) {
        eprintln!("Please specify which assets to compile. Use -h for help.");
        exit(1);
    }

    // TODO: Granular serialization support
    let mut ser = sundile_assets::Serializer::new()
        .with_asset_directory(args.in_path)
        .with_out_path(args.out_path);

    if args.shaders || args.all {
        ser = ser.with_mapper("shaders", types::shaders::Mapper::new());
    }
    if args.models || args.all {
        ser = ser.with_mapper("models", types::models::Mapper::new());
    }
    if args.textures || args.all {
        ser = ser.with_mapper("textures", types::textures::Mapper::new());
    }
    if args.fonts || args.all {
        ser = ser.with_mapper("fonts", types::fonts::Mapper::new());
    }

    ser.serialize();
}

#[test]
fn test() {
    use std::process::{Command, Stdio};
    Command::new("./target/debug/sundile_serialize_assets")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
