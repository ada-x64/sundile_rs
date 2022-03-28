# sundile_rs

![3-23-33 debug_gui and simplex_noise](https://user-images.githubusercontent.com/41482263/160485247-2a29fbbb-2f11-4faf-8d1b-2e340250ac71.PNG)
*Example image showing the Debug GUI and a terrain chunk rendered with [slib_terrain](https://github.com/ada-rose-dev/quell_lib/tree/main/terrain)*

Sundile is a data-driven game engine/framework written in rust. It continues the work done in [sundile (C++)](https://github.com/ada-rose-dev/sundile).

As of now, it is *not* available on crates.io, and I am *not* practicing good SemVer. This will change once the library has stabilized.

For an example of usage, see [Quell](https://github.com/ada-rose-dev/quell).

## Crates

### [sundile_assets](https://github.com/ada-rose-dev/sundile_rs/tree/main/assets)
This is a general-purpose, extensible asset loading library. It can load data at runtime, and can (de)serialize data at compile time.

### [sundile_graphics](https://github.com/ada-rose-dev/sundile_rs/tree/main/graphics)
This is essentially a WGPU wrapper. Its primary function is to provide data types and an abstract Render Target.

### [sundile_scripting](https://github.com/ada-rose-dev/sundile_rs/tree/main/scripting)
A deeply WIP scripting system designed to be used to implement an ECS and level system.

### [sundile_core](https://github.com/ada-rose-dev/sundile_rs/tree/main/core)
The core library, which brings all the above together into a Game struct.

### [sundile](https://github.com/ada-rose-dev/sundile_rs/tree/main/frontend)
The front-end library for Sundile. This library handles engine creation and implements a Debug GUI. Additionally, it provides an interface for extensions.
