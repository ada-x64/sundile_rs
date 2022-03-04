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
* Build script calls assets::load().
* `data.bin` is generated and placed with source files.
* Macro loads `data.bin` at compile time, embedding the data in the application.