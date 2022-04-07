use std::io::{Read, Write};
use anyhow::*;

fn write_zip<W: std::io::Write + std::io::Seek>(dir: String, zip: &mut zip::ZipWriter<W>) -> Result<()> {
    let pkg_dir = std::fs::read_dir(&dir)?;
    zip.add_directory(&dir, Default::default())?;

    for f in pkg_dir {
        let entry = f?;
        if entry.path().is_dir() {
            let new_dir = dir.clone() + "/" + entry.path().file_name().unwrap().to_str().unwrap();
            write_zip(new_dir, zip)?;
        }
        else {
            let file_name = entry.file_name().into_string().unwrap();
            zip.start_file(format!("{}/{}", dir, file_name), Default::default())?;
            let mut buf = Vec::new();
            std::fs::File::open(entry.path())?.read_to_end(&mut buf)?;
            zip.write_all(&buf[..])?;
        }
    }
    Ok(())
}

fn build<'a>() -> Result<()> {
    // TODO: Take in build and out directories via cmdline argments.
    let mut args = std::env::args();
    args.next();

    match std::process::Command::new("wasm-pack")
        .arg("build")
        .arg("--target=web")
        .arg("--out-dir=./server/pkg")
        .arg("--out-name=target")
        .args(args)
        .spawn()?
        .wait_with_output()?
        .status.success() {
            true => {
                std::fs::File::create("server/index.html")?
                    .write(include_bytes!("index.html"))?;
            
                Ok(())
            },
            false => {
                Err(anyhow!("Failed to compile to WASM"))
            }
        }
}

fn main() -> Result<()> {
    println!("Writing to ./server...");
    build()?;

    println!("Writing to ./itch-pack.zip...");
    let archive = std::fs::File::create("./itch-pack.zip")?;
    let mut zip = zip::ZipWriter::new(archive);
    match write_zip("./server".into(), &mut zip) {
        Err(e) => {
            // Finish and remove file. Don't want a half-packaged thing.
            zip.finish()?;
            std::fs::remove_file("./itch-pack.zip")?;
            Err(e)
        }
        _ => {
            zip.finish()?;
            println!("Archive successfully created!");
            println!("(Tip: Run `npx http-server ./server/ -o` to test locally.)");
            Ok(())
        }
    }
}