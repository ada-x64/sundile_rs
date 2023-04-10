# asests

This library handles asset loading and is intended to be run at compile time.

Desired Functionality:
* Extensible.
* Reads in a package tracking file with last-modified dates for each tracked asset.
* Reads in fresh asset files from disk and extends / modifies already saved `data.bin`.
* Compresses assets.
* Exports the files to `data.bin`, which will be read in at compile time by sundile_core using a macro.
* Provides this functionality to sundile_core.

Timeline:
* Build script creates a `Serializer` and defines how raw assets are to be compiled.
    * The serializer takes functions that turn a pathbuf into a RawAssetMap.
    * These RawAssetMaps are compiled into a RawAssetTypeMap.
    * It then serializes these maps using bincode.
* main.rs loads serialized data into the game using `include_bytes!`
* The serialized data is internally deserialized with a `Deserializer`.
    * The deserializer converts the bincode data back into a RawAssetTypeMap.
    * The RawAssetTypeMap recursively converts its raw data into the specified types.
    * Those types are then stored in a plain AssetTypeMap, which can be used by the game engine.