use crate::prelude::*;
use cgmath::InnerSpace;
use cgmath::*;
use serde::*;
use std::{path::Path, rc::Rc};
use thiserror::Error;
use tobj::LoadOptions;
use wgpu::util::DeviceExt;

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

/// A Material is a collection of textures used on a model.
pub struct Material {
    pub diffuse_texture: Rc<TextureWrapper>,
    pub normal_texture: Rc<TextureWrapper>,
    pub bind_group: wgpu::BindGroup,
}
impl Material {
    /// Creates a new [Material].
    pub fn new(
        label: Option<&str>,
        diffuse_texture: Rc<TextureWrapper>,
        normal_texture: Rc<TextureWrapper>,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_layout,
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
            label,
        });
        Self {
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}
impl core::fmt::Debug for Material {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.debug_list().entry(&self.bind_group).finish()
    }
}

/// TODO: FIXME: This only takes diffuse and normal textures.
/// Fix this so that it can deal with all of them.
/// A terrible little builder struct so we can have a portable intermediary format.
/// Note that this is only useful if you cannot directly pass the [Texture]s into [Material]::new().
#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialBuilder {
    diffuse_texture: Vec<u8>,
    normal_texture: Vec<u8>,
    label: Option<String>,
}
impl MaterialBuilder {
    /// Creates a new MaterialBuilder. Note that this takes textures as bytes.
    pub fn new(label: Option<String>, diffuse_texture: Vec<u8>, normal_texture: Vec<u8>) -> Self {
        Self {
            diffuse_texture,
            normal_texture,
            label,
        }
    }
    /// Builds the material. This will fail if the textures cannot be created from the passed-in bits.
    /// You might want to pass the newly-created Rc<Texture>'s to an AssetTypeMap.
    pub fn build(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Material {
        let name = self.label.unwrap_or("unnamed mesh".to_string());
        let diffuse_texture = Rc::new(
            TextureWrapper::from_bytes(
                device,
                queue,
                &self.diffuse_texture[..],
                &*format!("{} Diffuse", &name),
                false,
            )
            .unwrap(),
        );
        let normal_texture = Rc::new(
            TextureWrapper::from_bytes(
                device,
                queue,
                &self.normal_texture[..],
                &*format!("{} Normal", &name),
                true,
            )
            .unwrap(),
        );
        Material::new(
            Some(&*name),
            diffuse_texture,
            normal_texture,
            device,
            texture_layout,
        )
    }
}

/// A mesh describes part of the geometry of a model.
#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

/// A struct for creating meshes. Generally follows the builder pattern, but [generate] does not consume self.
/// This allows you to keep the vertices and indices for later use or modification.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshBuilder {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    name: Option<String>,
    material_id: Option<usize>,
    calculate_tangents: bool,
}
impl MeshBuilder {
    /// Creates a new MeshBuilder.
    pub fn new(vertices: Vec<ModelVertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            name: None,
            material_id: None,
            calculate_tangents: false,
        }
    }
    /// Allows you to replace the currently existing vertex array while conforming to builder pattern.
    pub fn with_vertices(mut self, vertices: Vec<ModelVertex>) -> Self {
        self.vertices = vertices;
        self
    }
    /// Allows you to replace the currently exiting index array while conforming to builder pattern.
    pub fn with_indices(mut self, indices: Vec<u32>) -> Self {
        self.indices = indices;
        self
    }
    /// Adds a name to the mesh, used for debugging. Defaults to "unnamed mesh".
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    /// Adds a material_id to the mesh. This needs to be correlated to your model's materials. This defaults to 0.
    pub fn with_material_id(mut self, material_id: usize) -> Self {
        self.material_id = Some(material_id);
        self
    }
    /// Calculates tangents and bitangents for the mesh.
    pub fn with_tangents(mut self, calculate_tangents: bool) -> Self {
        self.calculate_tangents = calculate_tangents;
        self
    }
    /// Generates a [Mesh]. Does *not* consume self.
    pub fn generate(&mut self, device: &wgpu::Device) -> Mesh {
        if self.calculate_tangents {
            self.calculate_tangents();
        }
        let name = self.name.clone().unwrap_or("unnamed mesh".to_string());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Mesh {
            name,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            material: self.material_id.unwrap_or(0),
        }
    }
    /// Builds a [Mesh], consuming self.
    pub fn build(mut self, device: &wgpu::Device) -> Mesh {
        if self.calculate_tangents {
            self.calculate_tangents();
        }
        let name = self.name.unwrap_or("unnamed mesh".to_string());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Mesh {
            name,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            material: self.material_id.unwrap_or(0),
        }
    }

    fn calculate_tangents(&mut self) {
        let (vertices, indices) = (&mut self.vertices, &self.indices);
        let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();

        for c in indices.chunks(3) {
            let v0 = vertices[c[0] as usize];
            let v1 = vertices[c[1] as usize];
            let v2 = vertices[c[2] as usize];

            let pos0: cgmath::Vector3<_> = v0.position.into();
            let pos1: cgmath::Vector3<_> = v1.position.into();
            let pos2: cgmath::Vector3<_> = v2.position.into();

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

            vertices[c[0] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
            vertices[c[1] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
            vertices[c[2] as usize].tangent =
                (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();

            vertices[c[0] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
            vertices[c[1] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
            vertices[c[2] as usize].bitangent =
                (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

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
    }
}

/// The ModelInstance struct defines the position and rotation of a model instance.
#[derive(Debug)]
pub struct ModelInstance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}
impl ModelInstance {
    /// Creates a new instance at the given position and rotation.
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }
    /// Creates an instance at the origin with no rotation.
    pub fn at_origin() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
        }
    }
    /// Converts this easy-to-manipulate struct to the POD version used in shaders, [InstanceRaw].
    pub fn as_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position) * Matrix4::from(self.rotation)).into(),
            normal: Matrix3::from(self.rotation).into(),
        }
    }
    // pub fn from_transform(t: sundile_scripting::components::Transform) -> Self {
    // 	Self {
    // 		position: Vector3::new(t.x, t.y, t.z),
    // 		rotation: Quaternion::from(Euler::new(Deg(t.yaw), Deg(t.pitch), Deg(t.roll)))
    // 	}
    // }
}

/// The InstanceRaw struct is a POD description of an instance.
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

/// An InstanceCache holds all the instancing information for a particular model.
#[derive(Debug)]
pub struct InstanceCache {
    instances: Vec<ModelInstance>,
    buffer: Option<wgpu::Buffer>,
    dirty: bool,
    // ranges: Vec<Range>,
}
impl InstanceCache {
    /// Create a new InstanceCache.
    pub fn new() -> Self {
        Self {
            instances: vec![],
            buffer: None,
            dirty: true,
            // ranges: vec![],
        }
    }
    /// Insert a new [ModelInstance] to the cache.
    pub fn insert(&mut self, instance: ModelInstance) {
        self.dirty = true;
        self.instances.push(instance);
    }
    /// Remove all [ModelInstance]s from the cache.
    pub fn clear(&mut self) {
        self.dirty = true;
        self.instances.clear();
    }
    /// Updates the cache's internal buffer if necessary.
    pub fn update(&mut self, device: &wgpu::Device) {
        if self.dirty {
            self.buffer = Some(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(
                        &self
                            .instances
                            .iter()
                            .map(ModelInstance::as_raw)
                            .collect::<Vec<_>>(),
                    ),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            );
        }
        self.dirty = false;
    }
    /// Sets the ranges of instances to be displayed. This is used so as not to render any instances that are off-screen.
    pub fn set_ranges(&mut self) {
        // This should allow you to set which instances to render.
        // Possibly add helpers for all, none
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Failed to load model: `{0}`")]
    Load(#[from] tobj::LoadError),
    #[error("Unable to open model at path `{0}`")]
    Path(String),
    #[error("Unable to load model texture: `{0}`")]
    Texture(#[from] TextureError),
}

/// A Model is a collection of [Mesh]s and [Material]s. It is displayed in the world using [ModelInstance]s.
/// TODO: Make the correlation between meshes and materials embedded in the code instead of relying on indexing.
/// In particular, there is a one-to-many relationship from materials to meshes. Could be:
/// HashMap<Rc<Material>, Vec<Rc<Mesh>>>
#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Rc<Mesh>>,
    pub materials: Vec<Rc<Material>>,
    pub instance_cache: InstanceCache,
}
impl Model {
    /// This function loads in a model from an OBJ file.
    pub fn load<P: AsRef<Path>>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        path: P,
    ) -> Result<Self, ModelError> {
        let (obj_models, obj_materials) = tobj::load_obj(
            path.as_ref(),
            &LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        let obj_materials = obj_materials?;

        let containing_folder = path
            .as_ref()
            .parent()
            .ok_or_else(|| ModelError::Path(path.as_ref().to_string_lossy().to_string()))?;

        let mut materials = Vec::new();
        for mat in obj_materials {
            let diffuse_path = mat.diffuse_texture;
            let diffuse_texture = Rc::new(TextureWrapper::load(
                device,
                queue,
                containing_folder.join(diffuse_path),
                false,
            )?);

            let normal_path = mat.normal_texture;
            let normal_texture = Rc::new(TextureWrapper::load(
                device,
                queue,
                containing_folder.join(normal_path),
                true,
            )?);

            materials.push(Rc::new(Material::new(
                Some(mat.name.as_str()),
                diffuse_texture,
                normal_texture,
                &device,
                texture_layout,
            )));
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
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    tangent: [0.0; 3], //calculated below
                    bitangent: [0.0; 3],
                });
            }

            meshes.push(Rc::new(
                MeshBuilder::new(vertices, m.mesh.indices.clone())
                    .with_name(path.as_ref().display().to_string())
                    .with_material_id(m.mesh.material_id.unwrap_or(0))
                    .with_tangents(true)
                    .generate(device),
            ));
        }

        Ok(Self {
            meshes,
            materials,
            instance_cache: InstanceCache::new(),
        })
    }

    /// This function renders all of the model's instances to the screen.
    pub fn render<'r>(
        &'r self,
        render_pass: &mut wgpu::RenderPass<'r>,
        camera_bind_group: &'r wgpu::BindGroup,
        light_bind_group: &'r wgpu::BindGroup,
    ) {
        render_pass.set_vertex_buffer(1, self.instance_cache.buffer.as_ref().unwrap().slice(..));
        for mesh in &self.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.materials[mesh.material].bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);
            render_pass.set_bind_group(2, light_bind_group, &[]);
            render_pass.draw_indexed(
                0..mesh.num_elements,
                0,
                0..self.instance_cache.instances.len() as u32,
            ); //TODO: Add actual ranges.
        }
    }
}
