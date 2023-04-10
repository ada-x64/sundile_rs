use serde::*;
use std::fs::*;
use std::io::Read;
use std::path::*;
use std::rc::Rc;

use crate::*;
use sundile_graphics::*;

#[derive(Serialize, Deserialize)]
pub struct ModelData {
    pub material_builders: Vec<MaterialBuilder>,
    pub mesh_builders: Vec<MeshBuilder>,
}

impl RawAsset<Model> for ModelData {
    fn from_disk(path: &PathBuf) -> Self {
        let (obj_models, obj_materials) = tobj::load_obj(
            &path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load .obj");

        let obj_materials = obj_materials.expect("Failed to unwrap materials");

        let dir = path.parent().unwrap();
        let mut material_builders = Vec::new();
        for mat in obj_materials {
            //TODO: Probably compress these.
            //FIXME: This only takes diffuse and normal textures.
            //There are so many other kinds of texture.
            //Fix this so that it can deal with all of them.
            let mut diffuse_texture = Vec::<u8>::new();
            let mut file = File::open(dir.join(mat.diffuse_texture)).unwrap();
            file.read_to_end(&mut diffuse_texture).unwrap();

            let mut normal_texture = Vec::<u8>::new();
            let mut file = File::open(dir.join(mat.normal_texture)).unwrap();
            file.read_to_end(&mut normal_texture).unwrap();

            material_builders.push(MaterialBuilder::new(
                Some(mat.name),
                diffuse_texture,
                normal_texture,
            ));
        }

        let mut mesh_builders = Vec::new();
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

            mesh_builders.push(
                MeshBuilder::new(vertices, m.mesh.indices)
                    .with_material_id(m.mesh.material_id.unwrap_or(0))
                    .with_name(m.name)
                    .with_tangents(true),
            );
        }

        Self {
            mesh_builders,
            material_builders,
        }
    }

    fn to_asset(self, builder: &AssetBuildTarget) -> Model {
        let (device, queue, texture_layout) =
            (builder.device, builder.queue, builder.texture_layout);

        let mut materials = vec![];
        for builder in self.material_builders {
            materials.push(Rc::new(builder.build(device, queue, texture_layout)));
        }

        let mut meshes = vec![];
        for mut builder in self.mesh_builders {
            meshes.push(Rc::new(builder.generate(&device)));
        }

        Model {
            meshes,
            materials,
            instance_cache: InstanceCache::new(),
        }
    }
}

pub type Mapper = std::collections::HashMap<String, ModelData>;
impl RawAssetMapper for Mapper {
    fn load(&mut self, asset_dir: &PathBuf) {
        crate::util::generic_load(self, asset_dir, "models", "obj");
    }
    fn to_asset_map(self: Box<Self>, builder: &AssetBuildTarget) -> AssetMap {
        crate::util::generic_to_asset_map(*self, builder)
    }
    fn load_bin_map(&mut self, bin_map: BincodeAssetMap) {
        crate::util::generic_load_bin_map(self, bin_map);
    }
    fn to_bin_map(self: Box<Self>) -> BincodeAssetMap {
        crate::util::generic_to_bin_map(*self)
    }
}

