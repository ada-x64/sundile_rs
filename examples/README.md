# examples

This crate contains a variety of examples that show off the capabilities of Sundile.

## Compiling

Due to the way WASM builds work, this crate uses feature gates instead of the built-in `cargo example` command.

Before you can run an example that contains assets, you will need to serialize them.
This can be done with `tools/sundile_serialize_assets`.

To install all the tools, run

```
cargo install --path $PATH_TO_WORKSPACE_ROOT/tools
```

In order to serialize e.g. the `model_import` example, run this command:

```
sundile_serialize_assets -i ./src/model_import/assets -o ./src/model_import -m
```

You can run the examples locally using a basic Cargo command, e.g.

```
cargo run -F model_import
```

In order to build the examples for the web, you'll need to package the library using `tools/sundile_pack`.

```
sundile_pack --dev -- -F model_import
```
