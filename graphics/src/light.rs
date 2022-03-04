use wgpu::util::DeviceExt;
use wgpu::*;

pub const NUM_LIGHTS: usize = 10;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
	pub position: [f32; 3],
	pub _padding: u32,
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
    fn new(position: [f32;3], color: [f32;4]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
        }
    }
}

pub struct LightWrapper {
	lights: [LightUniform; NUM_LIGHTS],
    used_lights: usize,
	pub bind_group_layout: BindGroupLayout,
}

impl LightWrapper {
    pub fn new(device: &Device,) -> Self {
        let bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("Light Buffer Layout"),
            }
        );

        Self {
            lights: [LightUniform::default(); NUM_LIGHTS],
            used_lights: 0,
            bind_group_layout,
        }
    }
    /// Adds a point light uniform for this pass.
    pub fn add_light(&mut self, position: [f32; 3], color: [f32;4]) -> Result<(), &str> {
        if self.used_lights >= NUM_LIGHTS {
            return Err("Exceeded maximum number of lights!");
        }
        self.lights[self.used_lights] = LightUniform::new(position, color);
        self.used_lights += 1;
        Ok(())
    }
    /// Gets the current light bind group and clears the light uniforms for this pass.
    pub fn get_bind_group(&mut self, device: &wgpu::Device) -> BindGroup {
        // let mut buffers = vec![];
        // for i in 0..NUM_LIGHTS {
        //     buffers.push(device.create_buffer_init(
        //         &util::BufferInitDescriptor {
        //             label: Some(format!("Light VB {}", i).as_str()),
        //             contents: bytemuck::cast_slice(&[self.lights[i]]),
        //             usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        //         }
        //     ));
        //     self.lights[i] = LightUniform::default();
        // }

        let buffer = device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some(format!("Lights Buffer").as_str()),
                contents: bytemuck::cast_slice(&[self.lights]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }
        );

        device.create_bind_group(
            &BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }
        )
    }
}