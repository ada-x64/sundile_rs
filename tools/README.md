# tools

These are tools that do not require the sundile library, but are nice to have alongside it.

## sundile_pack

This little bin will take your project, compile it to WASM, and package it into an [itch.io](itch.io)-friendly ZIP file.

In order for this to work, you will need to satisfy the following requirements:

1. [wasm-pack](https://rustwasm.github.io/wasm-pack/) must be installed.
2. You must separate your project into a "cdylib" style crate. See the wasm-pack documents for more details.
3. You must create a function tagged with `#[wasm_bindgen(start)]`. See the examples crate for ... examples.

## sundile_serialize_assets

This is a binary version of the assets serializer. This tool is useful when you want to compress your assets.
If you don't want to run the binary, you can always run the serializer during the build process.
