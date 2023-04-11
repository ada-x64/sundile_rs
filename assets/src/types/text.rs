use std::{fs::File, io::Read};

use sundile_graphics::TextBlock;

use crate::RawAsset;

// WWID:
// Implement Text as an Asset type so we can integrate it with SceneBuilder.
// This will be used for GUI text and other things with Renderer2D.
// Speaking of, that should be made into an asset and taken out of the main render pipeline.
//
// Performance is overall pretty atrocious as well. Will need to overhaul asset storage
// system and will need to multithread the core library.
//
// Do some profilng (flamegraph, internal profiling?) to get an idea of what's going on.
// TODO: ModelInstance::as_raw() is taking up a HUGE amount of CPU power.
// Cache the raw instance instead of calculating it every frame.

type TextData = String;

impl RawAsset<TextBlock> for TextData {
    /// Simply reads a file into a UTF-8 string.
    fn from_disk(path: &std::path::PathBuf) -> Self {
        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();
        buf
    }

    fn to_asset<'f>(self, _render_target: &crate::AssetBuildTarget<'f>) -> TextBlock {
        TextBlock::new(self.clone())
    }
}
