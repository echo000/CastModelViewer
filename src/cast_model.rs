use porter_cast::{CastId, CastNode};
use porter_math::{Quaternion, Vector2, Vector3};
use porter_model::{
    Bone, Face, FaceBuffer, Material, MaterialTextureRef, MaterialTextureRefUsage, Mesh, Model,
    Skeleton, VertexBuffer,
};
use porter_texture::{Image, ImageFileType};
use rayon::prelude::*;
use std::path::Path;

pub fn load_model_images(model: &Model, file_name: &Path) -> Vec<Option<Image>> {
    model
        .materials
        .par_iter()
        .map(|mats| {
            // Find the first texture with matching usage
            let texture = mats.textures.iter().find(|images| {
                images.texture_usage == MaterialTextureRefUsage::Diffuse
                    || images.texture_usage == MaterialTextureRefUsage::Albedo
            });
            if let Some(images) = texture {
                let directory = file_name.parent().unwrap_or(Path::new("."));
                let f = directory.join(&images.file_name);
                let image_file_type = match f.extension().and_then(|ext| ext.to_str()) {
                    Some("png") => ImageFileType::Png,
                    Some("dds") => ImageFileType::Dds,
                    Some("tiff") => ImageFileType::Tiff,
                    Some("tga") => ImageFileType::Tga,
                    Some(ext) => {
                        eprintln!("Unsupported file extension: {ext}");
                        ImageFileType::Dds
                    }
                    None => {
                        eprintln!("File has no extension");
                        ImageFileType::Dds
                    }
                };
                match Image::load(f, image_file_type) {
                    Ok(image) => Some(image),
                    Err(err) => {
                        eprintln!("Failed to load image: {}: {:?}", &images.file_name, err);
                        None
                    }
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn process_model_node(model_node: &CastNode) -> Option<Model> {
    let mut model = Model::new();
    model.skeleton = model_node
        .children_of_type(CastId::Skeleton)
        .map(process_skeleton_node)
        .next()
        .unwrap_or_else(Skeleton::default);
    process_material_nodes(model_node, &mut model);
    process_mesh_nodes(model_node, &mut model);
    Some(model)
}

fn process_skeleton_node(skeleton_node: &CastNode) -> Skeleton {
    let bones = skeleton_node
        .children_of_type(CastId::Bone)
        .map(process_bone_node)
        .collect();
    let mut skeleton = Skeleton::new();
    skeleton.bones = bones;
    skeleton
}

fn process_bone_node(bone_node: &CastNode) -> Bone {
    Bone {
        name: bone_node
            .property("n")
            .and_then(|p| p.values::<String>().next()),
        parent: bone_node
            .property("p")
            .and_then(|p| p.values::<u32>().next())
            .map(|v| v as i32)
            .unwrap_or(-1),
        local_position: bone_node
            .property("lp")
            .and_then(|p| p.values::<Vector3>().next()),
        local_rotation: bone_node
            .property("lr")
            .and_then(|p| p.values::<Quaternion>().next()),
        local_scale: bone_node
            .property("s")
            .and_then(|p| p.values::<Vector3>().next()),
        world_position: bone_node
            .property("wp")
            .and_then(|p| p.values::<Vector3>().next()),
        world_rotation: bone_node
            .property("wr")
            .and_then(|p| p.values::<Quaternion>().next()),
        world_scale: bone_node
            .property("s")
            .and_then(|p| p.values::<Vector3>().next()),
    }
}

fn process_material_nodes(model_node: &CastNode, model: &mut Model) {
    let new_materials: Vec<Material> = model_node
        .children_of_type(CastId::Material)
        .map(|child_node| {
            let name = child_node
                .property("n")
                .and_then(|p| p.values::<String>().next())
                .unwrap_or_default();

            let mut material = Material::new(name);

            let albedo_hash = child_node
                .property("albedo")
                .and_then(|p| p.values::<u64>().next())
                .or_else(|| {
                    child_node
                        .property("diffuse")
                        .and_then(|p| p.values::<u64>().next())
                })
                .unwrap_or(0);

            if let Some(albedo) = child_node.child_by_hash(albedo_hash) {
                let file_name = albedo
                    .property("p")
                    .and_then(|p| p.values::<String>().next())
                    .unwrap_or_default();

                let texture_ref = MaterialTextureRef {
                    file_name: file_name.to_string(),
                    texture_usage: MaterialTextureRefUsage::Albedo,
                    texture_alias: "".to_string(),
                    texture_hash: albedo_hash,
                };
                material.textures.push(texture_ref);
            }

            material
        })
        .collect();

    model.materials.extend(new_materials);
}

fn process_mesh_nodes(model_node: &CastNode, model: &mut Model) {
    // Gather all mesh nodes first
    let mesh_nodes: Vec<&CastNode> = model_node.children_of_type(CastId::Mesh).collect();

    let meshes: Vec<Mesh> = mesh_nodes
        .par_iter()
        .map(|child_node| {
            let uv_layers = child_node
                .property("ul")
                .and_then(|p| p.values::<u32>().next())
                .unwrap_or(0);

            let weight_influence = child_node
                .property("mi")
                .and_then(|p| p.values::<u32>().next())
                .unwrap_or(0);

            //This may be the worst thing I've ever seen???
            let material_index = child_node
                .property("m")
                .and_then(|p| p.values::<u64>().next())
                .and_then(|hash| model_node.child_by_hash(hash))
                .and_then(|mat_node| {
                    mat_node
                        .property("n")
                        .and_then(|p| p.values::<String>().next())
                })
                .and_then(|mat_name| model.materials.iter().position(|mat| mat.name == mat_name));

            let mut vertex_buffer = VertexBuffer::builder()
                .colors(0)
                .uv_layers(uv_layers as usize)
                .maximum_influence(weight_influence as usize)
                .build();

            // Vertex Positions
            if let Some(vp_property) = child_node.property("vp") {
                for pos in vp_property.values::<Vector3>() {
                    vertex_buffer.create().set_position(pos);
                }
            }

            // Normals
            if let Some(vn_property) = child_node.property("vn") {
                for (i, n) in vn_property.values::<Vector3>().enumerate() {
                    vertex_buffer.vertex_mut(i).set_normal(n);
                }
            }

            // UV0
            if let Some(uv0_property) = child_node.property("u0") {
                for (i, uv) in uv0_property.values::<Vector2>().enumerate() {
                    vertex_buffer.vertex_mut(i).set_uv(0, uv);
                }
            }

            // UV1
            if let Some(uv1_property) = child_node.property("u1") {
                for (i, uv) in uv1_property.values::<Vector2>().enumerate() {
                    vertex_buffer.vertex_mut(i).set_uv(1, uv);
                }
            }

            // Faces
            let mut face_buffer = FaceBuffer::new();
            if let Some(f_property) = child_node.property("f") {
                let indices: Vec<u32> = f_property.values::<u32>().collect();
                for chunk in indices.chunks_exact(3) {
                    face_buffer.push(Face::new(chunk[2], chunk[1], chunk[0]));
                }
            }

            Mesh {
                material: material_index,
                ..Mesh::new(face_buffer, vertex_buffer)
            }
        })
        .collect();

    model.meshes.extend(meshes);
}
