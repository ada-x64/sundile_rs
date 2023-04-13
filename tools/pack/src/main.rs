use clap::Parser;
use std::{
    fs::{read_dir, remove_file, File},
    io::{self, Read, Seek, Write},
};
use thiserror::Error;
use zip::{result::ZipError, ZipWriter};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "
sundile-pack is a command line tool for bundling Sundile projects as web-compatible WASM. \
In order for this to work, make sure you add the following to your Cargo.toml file:
          
[lib]
crate-type = [\"cdylib\", \"rlib\"]
          
If you want web and native builds, you will need to create a library and a binary package. \
An easy way to accomplish this is as follows:

tree ./pack-test
```
./pack-test
├── Cargo.toml
└── src
    ├── lib.rs
    └── main.rs
```

main.rs:
```
#[wasm_bindgen(start)]
pub fn main() {
    lib::doit();
}
```

lib.rs:
```
pub fn doit() {
    // actual application goes here
}
```
"
)]
struct Args {
    /// Directory of the crate to build.
    #[arg(long, short, default_value = "./")]
    target_directory: String,
    /// Directory to output the built package.
    /// This is relative to the target directory!
    #[arg(long, short = 'o', default_value = "server/pkg")]
    out_dir: String,

    /// Create a development build. Enable debug info, and disable optimizations.
    #[arg(long)]
    dev: bool,
    /// Create a release build. Enable optimizations and disable debug info.
    #[arg(long)]
    release: bool,
    /// Create a profiling build. Enable optimizations and debug info.
    #[arg(long)]
    profiling: bool,

    /// Create a .zip archive for easy upload to sites like itch.io.
    #[arg(long)]
    zip: bool,

    // /// Arguments to be passed to wasm-pack.
    // #[arg(long,short)]
    // wasm_pack_args: Option<String>,
    /// Arguments to be passed to Cargo for building.
    #[arg(last = true)]
    cargo_args: Vec<String>,
}

#[derive(Error, Debug)]
enum BuildError {
    #[error("Unable to compile WASM")]
    WasmBuild,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Zip error: {0}")]
    Zip(#[from] ZipError),
    #[error("{0}")]
    Custom(String),
}

// Calls wasm-pack and adds the included index.html.
fn build<'a>(args: &Args) -> Result<(), BuildError> {
    let mut cmd = std::process::Command::new("wasm-pack");
    cmd.arg("build")
        .arg("--target=web")
        .arg(format!("--out-dir={}", args.out_dir))
        .arg("--out-name=target");

    if args.dev {
        cmd.arg("--dev");
    }
    if args.release {
        cmd.arg("--release");
    }
    if args.profiling {
        cmd.arg("--profiling");
    }
    if !(args.dev || args.release || args.profiling) {
        println!("WARNING: wasm-bindgen compiles release builds by default.");
        println!("To enable development builds, pass '--dev'");
    }

    cmd.arg(args.target_directory.clone())
        .args(args.cargo_args.clone());

    match cmd.spawn()?.wait_with_output()?.status.success() {
        true => {
            File::create(format!(
                "{}/{}/index.html",
                args.target_directory, args.out_dir
            ))?
            .write(include_bytes!("index.html"))?;

            Ok(())
        }
        false => Err(BuildError::WasmBuild),
    }
}

// Recursively zips the package directory for easy upload to e.g. itch.io
fn write_zip<W: Write + Seek>(dir: &String, zip: &mut ZipWriter<W>) -> Result<(), BuildError> {
    let pkg_dir = read_dir(dir)?;
    zip.add_directory(dir, Default::default())?;

    for file in pkg_dir {
        let entry = file?;
        if entry.path().is_dir() {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or(BuildError::Custom(format!(
                    "Unable to compress {:?}: `file_name()` failed",
                    path
                )))?;
            let new_dir = dir.clone() + "/" + file_name;
            write_zip(&new_dir, zip)?;
        } else {
            let file_name = entry.file_name().into_string().map_err(|e| {
                BuildError::Custom(format!("Unable to convert path string {:?} to UTF-8", e))
            })?;
            zip.start_file(format!("{}/{}", dir, file_name), Default::default())?;
            let mut buf = Vec::new();
            File::open(entry.path())?.read_to_end(&mut buf)?;
            zip.write_all(&buf[..])?;
        }
    }
    Ok(())
}

fn main() -> Result<(), BuildError> {
    let args = Args::parse();

    build(&args)?;

    let archive_path = format!("{}/{}/itch-pack.zip", args.target_directory, args.out_dir);
    if args.zip {
        let archive = File::create(&archive_path)?;
        let mut zip = ZipWriter::new(archive);
        match write_zip(&archive_path, &mut zip) {
            Ok(_) => {
                zip.finish()?;
                println!("Archive successfully created!");
            }
            Err(e) => {
                // Finish and remove file. Don't want a half-packaged thing.
                zip.finish()?;
                remove_file(&archive_path)?;
                return Err(e);
            }
        }
    };

    println!(
        "(Tip: Run `npx http-server {}/{} -o index.html` to test locally.)",
        args.target_directory, args.out_dir,
    );

    Ok(())
}
