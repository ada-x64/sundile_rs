# sundile_rs

![A field of rotated cubes with "Hello, Text!" written over it](./docs/Screenshot_20230411_170933.png)

Sundile is a rendering engine written in Rust. It utilizes WGPU as its graphics backend and is WASM compatible.

__Live examples are available [on my website.](https://cubething.dev/gfx)__

## Features

- Asset creation and importing
- Model rendering
- Quad rendering
- Text rendering
- Shader support
- Exports to WASM (runs in the browser)

## WIP

- Multithreading - core crate overhaul
- Models only support diffuse and specular textures for now
- Need to integrate with an ECS
