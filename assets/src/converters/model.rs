use std::collections::HashMap;
use anyhow::*;
use crate::internal::*;
use sundile_graphics::prelude::*;
use cgmath::InnerSpace;
use wgpu::util::DeviceExt;
use std::path::*;
use std::fs::*;
use std::io::Read;
use serde::*;

#[derive(Serialize, Deserialize)]
pub struct MaterialData {
    pub name: String,
    pub diffuse_texture: Vec<u8>,
    pub normal_texture: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct MeshData {
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub material: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ModelData {
    pub materials: Vec<MaterialData>,
    pub meshes: Vec<MeshData>,
}

//TODO: Clean this up using new Model fns
impl DataType<Model> for ModelData {
    fn load(path: &PathBuf) -> Self {
        let (obj_models, obj_materials) = tobj::load_obj(
            &path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        ).expect("Failed to load .obj");

        let obj_materials = obj_materials.expect("Failed to unwrap materials");

        let dir = path.parent().unwrap();
        let mut materials = Vec::new();
        for mat in obj_materials {
            //TODO: Probably compress these.
            let mut diffuse_texture = Vec::<u8>::new();
            let mut file = File::open(dir.join(mat.diffuse_texture)).unwrap();
            file.read_to_end(&mut diffuse_texture).unwrap();

            let mut normal_texture = Vec::<u8>::new();
            let mut file = File::open(dir.join(mat.normal_texture)).unwrap();
            file.read_to_end(&mut normal_texture).unwrap();

            materials.push(MaterialData {
                name: mat.name,
                diffuse_texture,
                normal_texture,
            });
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

            let indices = &m.mesh.indices;
            let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();

            //Calculate (bi)tangents
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

            meshes.push(MeshData {
                name: m.name,
                vertices,
                indices: m.mesh.indices,
                material: m.mesh.material_id.unwrap_or(0),
            });
        }

        Self {meshes, materials}
    }

    fn convert(self, render_target: &RenderTarget) -> Result<Model> {
        let (device, queue) = (&render_target.device, &render_target.queue);

        let mut materials = vec![];
        for data in self.materials {
            let diffuse_texture = Texture::from_bytes(device, queue, &data.diffuse_texture[..], format!("{} diffuse texture", data.name).as_str(), false).unwrap();
            let normal_texture = Texture::from_bytes(device, queue, &data.normal_texture[..], format!("{} normal texture", data.name).as_str(), true).unwrap();
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &Texture::get_bind_group_layout(&render_target.device),
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

            materials.push(Material {
                name: data.name,
                diffuse_texture,
                normal_texture,
                bind_group,
            });
        }

        let mut meshes = vec![];
        for data in self.meshes {
            let vertex_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", data.name)),
                    contents: bytemuck::cast_slice(&data.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
            let index_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", data.name)),
                    contents: bytemuck::cast_slice(&data.indices),
                    usage: wgpu::BufferUsages::INDEX,
                }
            );

            meshes.push(Mesh {
                name: data.name,
                vertex_buffer,
                index_buffer,
                num_elements: data.indices.len() as u32,
                material: data.material,
            });
        }

        Ok(Model{
            meshes,
            materials,
        })
    }
}


pub type DataMap = HashMap<String, ModelData>;
pub type EmbeddedMap = HashMap<String, Model>;

// impl Map<EmbeddedMap> for DataMap {
//     fn load(asset_dir: &PathBuf) -> DataMap {
        
//         echo("Loading models...");
//         let mut res = DataMap::new();
//         let mut path = asset_dir.to_owned();
//         path.push("models");

//         let dir = read_dir(path).unwrap().into_iter();
//         for subdir in dir {
//             let objs = read_dir(subdir.unwrap().path()).unwrap().into_iter().filter(
//                 |entry| {
//                     entry.as_ref().expect("bad directory entry!").path().extension().unwrap() == ".obj"
//                 });
//             for obj in objs {
//                 let obj = obj.unwrap();
//                 let name = obj.path().file_stem().unwrap().to_str().unwrap().to_string();
//                 res.insert(name, ModelData::load(&obj.path()));
//             }
//         }
//         res
//     }

//     fn convert(self, render_target: &RenderTarget) -> Result<EmbeddedMap> {
//         Ok(EmbeddedMap::from_iter(
//             self.into_iter().map(
//                 |(name, data)| -> (String, Model) {
//                     let data = data.convert(render_target)
//                         .expect(format!("Unable to convert {}.obj", &name).as_str());
//                     (name, data)
//                 }
//             )
//         ))
//     }
// }