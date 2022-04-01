use std::io::Write;
use anyhow::*;

fn write_zip<W: std::io::Write + std::io::Seek>(zip: &mut zip::ZipWriter<W>) -> Result<()> {
    let pkg_dir = std::fs::read_dir("./pkg")?;
    zip.add_directory("pkg", Default::default())?;

    for f in pkg_dir {
        use std::io::Read;
        let entry = f?;
        let file_name = entry.file_name().into_string().unwrap();
        let path = format!("pkg/{}", file_name);
        zip.start_file(path, Default::default())?;
        let mut buf = Vec::new();
        std::fs::File::open(entry.path())?.read_to_end(&mut buf)?;
        zip.write_all(&buf[..])?;

        if file_name.ends_with(".js") {
            let html = include_str!("index.html").replace("IMPORT_LOCATION", &format!("./pkg/{}",file_name));
            zip.start_file("index.html", Default::default())?;
            zip.write_all(html.as_bytes())?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // would be cool if there was a library for this
    // TODO: Take in build and out directories via cmdline argments.
    let mut args = std::env::args();
    args.next();
    let path = "./itch-pack.zip";
    let child = std::process::Command::new("wasm-pack").arg("build").arg("--target=web").args(args).spawn()?;
    let output = child.wait_with_output()?;
    if output.status.success() {
        println!("Writing to zip file...");
        let archive = std::fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(archive);
        match write_zip(&mut zip) {
            Err(e) => {
                // Finish and remove file. Don't want a half-packaged thing.
                zip.finish()?;
                std::fs::remove_file(path)?;
                Err(e)
            }
            _ => {
                zip.finish()?;
                println!("Archive successfully created!");
                Ok(())
            }
        }
    }
    else {
        Err(anyhow!("Failed to build WASM: {}", output.status))
    }
}