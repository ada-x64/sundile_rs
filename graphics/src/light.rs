use std::collections::HashMap;
use wgpu::util::DeviceExt;
use wgpu::*;

const NUM_LIGHTS: usize = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    _padding: u32,
    pub color: [f32; 4],
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            _padding: 0,
            color: [0.0; 4],
        }
    }
}
impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightBufferUniform {
    ambient: [f32; 4],
    lights: [LightUniform; NUM_LIGHTS],
}

#[derive(Debug)]
/// Wrapper for all light operations.
pub struct LightWrapper<'a> {
    uniform: LightBufferUniform,
    used_lights: usize,

    dirty: bool,
    buffer: Option<Buffer>,

    map: HashMap<&'a str, usize>, //Hashmap that points to stored light uniforms.
    pub bind_group_layout: BindGroupLayout,
}

impl<'a> LightWrapper<'a> {
    /// Creates a new light wrapper.
    /// Ambient light defaults to [1.0;4].
    pub fn new(device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Light Buffer Layout"),
        });

        Self {
            uniform: LightBufferUniform {
                ambient: [1.0; 4],
                lights: [LightUniform::default(); NUM_LIGHTS],
            },
            used_lights: 0,
            dirty: true,
            buffer: None,
            map: HashMap::new(),
            bind_group_layout,
        }
    }

    /// Sets the ambient color in RGBA, where A indicates strength
    pub fn set_ambient(&mut self, color: [f32; 4]) {
        self.dirty = true;
        self.uniform.ambient = color;
    }

    /// Adds a point light uniform.
    pub fn add_light(&mut self, name: &'a str, light: LightUniform) -> Result<(), &str> {
        if self.used_lights >= NUM_LIGHTS {
            return Err("Exceeded maximum number of lights!");
        }
        self.dirty = true;
        self.uniform.lights[self.used_lights] = light;
        self.map.insert(name, self.used_lights);
        self.used_lights += 1;
        Ok(())
    }

    /// Returns a non-mut LightUniform.
    pub fn get_light(&mut self, name: &'a str) -> Result<LightUniform, &str> {
        if !self.map.contains_key(name) {
            return Err("Cannot find light.");
        }
        Ok(self.uniform.lights[self.map[name]])
    }

    /// Replace the currently stored light uniform with a new one.
    pub fn update_light(&mut self, name: &'a str, light: LightUniform) -> Result<(), &str> {
        if !self.map.contains_key(name) {
            return Err("Cannot find light.");
        }
        self.dirty = true;
        self.uniform.lights[self.map[name]] = light;
        Ok(())
    }

    /// Removes the currently stored light uniform.
    pub fn remove_light(&mut self, name: &'a str) -> Result<(), &str> {
        if !self.map.contains_key(name) {
            return Err("Cannot find light.");
        }
        self.dirty = true;
        self.uniform.lights[self.map[name]] = LightUniform::default();
        // sort array
        self.used_lights -= 1;
        Ok(())
    }

    /// Gets the current light bind group and clears the light uniforms for this pass.
    /// Updates any dirty buffers.
    pub fn get_bind_group(&mut self, device: &wgpu::Device) -> BindGroup {
        if self.dirty {
            self.buffer = Some(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Lights buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }));
            self.dirty = false;
        }

        device.create_bind_group(&BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: self
                    .buffer
                    .as_ref()
                    .expect("Lights buffer not set!")
                    .as_entire_binding(),
            }],
            label: None,
        })
    }
}

