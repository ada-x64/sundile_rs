use crate::prelude::*;
use anyhow::*;
use std::ops::Range;
use std::path::Path;
use tobj::LoadOptions;
use wgpu::util::DeviceExt;
use cgmath::InnerSpace;
use serde::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture, //TODO: Probably shouldn't own these.
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}
impl Material {
    pub fn new(name: String, diffuse_texture: texture::Texture, normal_texture: texture::Texture, device: &wgpu::Device) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &Texture::get_bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: None,
        });
        Self {
            name,
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}
impl core::fmt::Debug for Material {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.debug_list().entry(&self.name).finish()
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct MeshBuilder {
    vertices: Vec<ModelVertex>,
    indices: Vec<u32>,
    name: Option<String>,
    material_id: Option<usize>,
}
impl MeshBuilder {
    pub fn new(vertices: Vec<ModelVertex>, indices: Vec<u32>, ) -> Self {
        Self {
            vertices,
            indices,
            name: None,
            material_id: None,
        }
    }
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    pub fn with_material_id(mut self, material_id: usize) -> Self {
        self.material_id = Some(material_id);
        self
    }
    pub fn with_tangents(mut self) -> Self { 
        let (vertices, indices) = (&mut self.vertices, &mut self.indices);
        let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();

        for c in indices.chunks(3) {
            let v0 = vertices[c[0] as usize];
            let v1 = vertices[c[1] as usize];
            let v2 = vertices[c[2] as usize];

            let pos0 : cgmath::Vector3<_> = v0.position.into();
            let pos1 : cgmath::Vector3<_> = v1.position.into();
            let pos2 : cgmath::Vector3<_> = v2.position.into();

            let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
            let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
            let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

            let delta_pos1 = pos1 - pos0;
            let delta_pos2 = pos2 - pos0;

            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            // Solving the following system of equations will
            // give us the tangent and bitangent.
            //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            // Luckily, the place I found this equation provided
            // the solution!

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

            vertices[c[0] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
            vertices[c[1] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
            vertices[c[2] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();

            vertices[c[0] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
            vertices[c[1] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
            vertices[c[2] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();
            
            // Used to average the tangents/bitangents
            triangles_included[c[0] as usize] += 1;
            triangles_included[c[1] as usize] += 1;
            triangles_included[c[2] as usize] += 1;
        }

        for (i, n) in triangles_included.into_iter().enumerate() {
            let denom = 1.0 / n as f32;
            let mut v = &mut vertices[i];
            v.tangent = (cgmath::Vector3::from(v.tangent) * denom)
                .normalize()
                .into();
            v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom)
                .normalize()
                .into();
        }

        self
    }
    pub fn build(self, device: &wgpu::Device) -> Mesh {
        let name = self.name.unwrap_or("unnamed mesh".to_string());

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", name)),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", name)),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        Mesh {
            name,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            material: self.material_id.unwrap_or(0),
        }
    }
}

//TODO: Decouple this so meshes can be rendered with arbitrary materials (?)
#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: P
    ) -> Result<Self> {
        let (obj_models, obj_materials) = tobj::load_obj(
            path.as_ref(),
&LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        let obj_materials = obj_materials?;

        let containing_folder = path.as_ref().parent().context("Directory has no parent")?;

        let mut materials = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            let diffuse_texture = texture::Texture::load(device, queue, containing_folder.join(diffuse_path), false)?;

            let normal_path = mat.normal_texture;
            let normal_texture = texture::Texture::load(device, queue, containing_folder.join(normal_path), true)?;

            materials.push(Material::new(mat.name, diffuse_texture, normal_texture, &device));
        }

        let mut meshes = Vec::new();
        for m in obj_models {
            let mut vertices = Vec::new();
            for i in 0..m.mesh.positions.len() / 3 {
                vertices.push(ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [
                        m.mesh.texcoords[i * 2],
                        m.mesh.texcoords[i * 2 + 1]
                    ],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    tangent: [0.0; 3], //calculated below
                    bitangent: [0.0; 3],
                });
            }

            meshes.push(MeshBuilder::new(vertices, m.mesh.indices.clone(),)
                .with_name(path.as_ref().display().to_string())
                .with_material_id(m.mesh.material_id.unwrap_or(0))
                .with_tangents()
                .build(device)
            );

            {
            // let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();

            // //Calculate (bi)tangents
            // for c in indices.chunks(3) {
            //     let v0 = vertices[c[0] as usize];
            //     let v1 = vertices[c[1] as usize];
            //     let v2 = vertices[c[2] as usize];

            //     let pos0 : cgmath::Vector3<_> = v0.position.into();
            //     let pos1 : cgmath::Vector3<_> = v1.position.into();
            //     let pos2 : cgmath::Vector3<_> = v2.position.into();

            //     let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
            //     let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
            //     let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

            //     let delta_pos1 = pos1 - pos0;
            //     let delta_pos2 = pos2 - pos0;

            //     let delta_uv1 = uv1 - uv0;
            //     let delta_uv2 = uv2 - uv0;

            //     // Solving the following system of equations will
            //     // give us the tangent and bitangent.
            //     //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
            //     //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
            //     // Luckily, the place I found this equation provided
            //     // the solution!

            //     let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
            //     let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
            //     let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

            //     vertices[c[0] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
            //     vertices[c[1] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
            //     vertices[c[2] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();

            //     vertices[c[0] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
            //     vertices[c[1] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
            //     vertices[c[2] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();
                
            //     // Used to average the tangents/bitangents
            //     triangles_included[c[0] as usize] += 1;
            //     triangles_included[c[1] as usize] += 1;
            //     triangles_included[c[2] as usize] += 1;
            // }

            // for (i, n) in triangles_included.into_iter().enumerate() {
            //     let denom = 1.0 / n as f32;
            //     let mut v = &mut vertices[i];
            //     v.tangent = (cgmath::Vector3::from(v.tangent) * denom)
            //         .normalize()
            //         .into();
            //     v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom)
            //         .normalize()
            //         .into();
            // }
            
            // let vertex_buffer = device.create_buffer_init(
            //     &wgpu::util::BufferInitDescriptor {
            //         label: Some(&format!("{:?} Vertex Buffer", path.as_ref())),
            //         contents: bytemuck::cast_slice(&vertices),
            //         usage: wgpu::BufferUsages::VERTEX,
            //     }
            // );
            // let index_buffer = device.create_buffer_init(
            //     &wgpu::util::BufferInitDescriptor {
            //         label: Some(&format!("{:?} Index Buffer", path.as_ref())),
            //         contents: bytemuck::cast_slice(&m.mesh.indices),
            //         usage: wgpu::BufferUsages::INDEX,
            //     }
            // );

            // meshes.push(Mesh {
            //     name: m.name,
            //     vertex_buffer,
            //     index_buffer,
            //     num_elements: m.mesh.indices.len() as u32,
            //     material: m.mesh.material_id.unwrap_or(0),
            // });
            }
        }

        Ok(Self {meshes, materials})
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a> where 'b: 'a, {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
    
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group,);
    }
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group, light_bind_group,);
        }
    }
}