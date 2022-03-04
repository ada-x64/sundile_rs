use crate::prelude::*;
use wgpu::util::*;
pub struct Instance {
	pub position: cgmath::Vector3<f32>,
	pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
	pub fn to_raw(&self) -> InstanceRaw {
		InstanceRaw {
			model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
			normal: cgmath::Matrix3::from(self.rotation).into(),
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
	pub model: [[f32; 4]; 4],
	pub normal: [[f32; 3]; 3],
}

impl Vertex for InstanceRaw {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &[
				// 4 vec4s = mat4x4
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 5,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
					shader_location: 6,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
					shader_location: 7,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
					shader_location: 8,
					format: wgpu::VertexFormat::Float32x4,
				},

				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
					shader_location: 9,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
					shader_location: 10,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
					shader_location: 11,
					format: wgpu::VertexFormat::Float32x3,
				},
			],
		}
	}
}

pub struct InstanceCache {
    instances: Vec<Instance>,
    buffer: Option<wgpu::Buffer>,
    stale: bool,
}

impl InstanceCache {
    pub fn new() -> Self {
        Self {
            instances: vec![],
            buffer: None,
            stale: true,
            // ranges: vec![],
        }
    }

    pub fn insert(&mut self, instance: Instance) {
        self.stale = false;
        self.instances.push(instance);
    }

    pub fn clear(&mut self) {
        self.stale = false;
        self.instances.clear();
    }

    pub fn update(&mut self, device: &wgpu::Device) {
        if !self.stale {
            self.buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>()),
                usage: wgpu::BufferUsages::VERTEX,
            }));
        }
        self.stale = true;
    }

    pub fn set_ranges(&mut self) {
        // This should allow you to set which instances to render.
        // Possibly add helpers for all, none
        todo!() 
    }

    pub fn render<'r>(&'r mut self,
        render_pass: &mut wgpu::RenderPass<'r>,
        model: &'r Model,
        camera_bind_group: &'r wgpu::BindGroup,
        light_bind_group: &'r wgpu::BindGroup,
    ) {
        render_pass.set_vertex_buffer(1, self.buffer.as_ref().unwrap().slice(..));
        render_pass.draw_model_instanced(model, 0..self.instances.len() as u32, &camera_bind_group, &light_bind_group);
    }
}