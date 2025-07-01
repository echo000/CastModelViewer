use crate::asset_manager::Asset;
use byteorder::{LittleEndian, ReadBytesExt};
use porter_math::{Quaternion, Vector2, Vector3, Vector4};
use porter_model::{
    Bone, Face, FaceBuffer, Material, MaterialTextureRef, MaterialTextureRefUsage, Mesh, Model,
    Skeleton, VertexBuffer,
};
use porter_texture::{Image, ImageFileType};
use porter_ui::PorterAssetStatus;
use porter_utils::{StringReadExt, StructReadExt};
use std::{
    collections::HashMap,
    io::{Read, Seek},
    path::Path,
};

#[derive(Clone, Debug)]
pub enum PropertyValue {
    Byte(u8),
    Short(u16),
    Int(u32),
    Long(u64),
    Float(f32),
    Double(f64),
    String(String),
    Vector2(Vector2),
    Vector3(Vector3),
    Vector4(Vector4),
}
impl PropertyValue {
    pub fn get_uint(&self) -> Option<u32> {
        match self {
            PropertyValue::Byte(value) => Some(*value as u32),
            PropertyValue::Short(value) => Some(*value as u32),
            PropertyValue::Int(value) => Some(*value),
            _ => None,
        }
    }
    pub fn get_int(&self) -> Option<i32> {
        match self {
            PropertyValue::Byte(value) => Some(*value as i32),
            PropertyValue::Short(value) => Some(*value as i32),
            PropertyValue::Int(value) => Some(*value as i32),
            _ => None,
        }
    }
    pub fn get_long(&self) -> Option<u64> {
        match self {
            PropertyValue::Byte(value) => Some(*value as u64),
            PropertyValue::Short(value) => Some(*value as u64),
            PropertyValue::Int(value) => Some(*value as u64),
            PropertyValue::Long(value) => Some(*value),
            _ => None,
        }
    }
    pub fn get_vector2(&self) -> Option<Vector2> {
        match self {
            PropertyValue::Vector2(vector) => Some(*vector),
            _ => None,
        }
    }
    pub fn get_vector3(&self) -> Option<Vector3> {
        match self {
            PropertyValue::Vector3(vector) => Some(*vector),
            _ => None,
        }
    }
    pub fn get_float(&self) -> Option<f32> {
        match self {
            PropertyValue::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn get_double(&self) -> Option<f64> {
        match self {
            PropertyValue::Double(value) => Some(*value),
            _ => None,
        }
    }
    pub fn get_quaternion(&self) -> Option<Quaternion> {
        match self {
            PropertyValue::Vector4(vector) => Some(Quaternion {
                x: vector.x,
                y: vector.y,
                z: vector.z,
                w: vector.w,
            }),
            _ => None,
        }
    }
    pub fn get_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default, Debug)]
pub struct CastHeader {
    pub magic: u32,
    pub version: u32,
    pub root_nodes: u32,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default, Debug)]
pub struct CastNodeHeader {
    pub identifier: u32,
    pub node_size: u32,
    pub node_hash: u64,
    pub property_count: u32,
    pub child_count: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Default, Debug)]
pub struct CastPropertyHeader {
    pub property_type: [u8; 2],
    pub name_length: u16,
    pub value_count: u32,
}

#[derive(Debug, Default, Clone)]
pub struct CastProperty {
    pub name: String,
    pub property_type: String,
    pub values: Vec<PropertyValue>,
}

impl CastProperty {
    pub fn new(name: String, property_type: String) -> Self {
        CastProperty {
            name,
            property_type,
            values: Vec::new(),
        }
    }

    pub fn load<R: Read + Seek>(reader: &mut R) -> Result<Self, std::io::Error> {
        let header: CastPropertyHeader = reader.read_struct()?;
        let name = reader.read_sized_string(header.name_length as usize, false)?;
        let property_type = String::from_utf8_lossy(&header.property_type).replace('\0', "");
        let mut property = CastProperty::new(name, property_type);

        for _ in 0..header.value_count {
            let value = property
                .read_property(reader, &property.property_type)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            property.values.push(value);
        }
        Ok(property)
    }

    pub fn read_property<R: Read + Seek>(
        &self,
        reader: &mut R,
        type_str: &str,
    ) -> Result<PropertyValue, std::io::Error> {
        match type_str {
            "b" => Ok(PropertyValue::Byte(reader.read_u8()?)),
            "h" => Ok(PropertyValue::Short(reader.read_u16::<LittleEndian>()?)),
            "i" => Ok(PropertyValue::Int(reader.read_u32::<LittleEndian>()?)),
            "l" => Ok(PropertyValue::Long(reader.read_u64::<LittleEndian>()?)),
            "f" => Ok(PropertyValue::Float(reader.read_f32::<LittleEndian>()?)),
            "d" => Ok(PropertyValue::Double(reader.read_f64::<LittleEndian>()?)),
            "s" => Ok(PropertyValue::String(reader.read_null_terminated_string()?)),
            "2v" => Ok(PropertyValue::Vector2(reader.read_struct::<Vector2>()?)),
            "3v" => Ok(PropertyValue::Vector3(reader.read_struct::<Vector3>()?)),
            "4v" => Ok(PropertyValue::Vector4(reader.read_struct::<Vector4>()?)),
            _ => Err(std::io::Error::other("invalid property type")),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct CastNode {
    pub identifier: u32,
    pub hash: u64,
    pub properties: HashMap<String, CastProperty>,
    pub child_nodes: Vec<CastNode>,
    pub parent_node: Option<u64>,
}

impl CastNode {
    pub fn new(identifier: u32, hash: u64, child_cap: usize) -> Self {
        CastNode {
            identifier,
            hash,
            properties: HashMap::new(),
            child_nodes: Vec::with_capacity(child_cap),
            parent_node: None,
        }
    }

    pub fn load<R: Read + Seek>(reader: &mut R) -> Result<Self, std::io::Error> {
        let header: CastNodeHeader = reader.read_struct()?;
        let mut node = CastNode::new(
            header.identifier,
            header.node_hash,
            header.child_count as usize,
        );

        for _ in 0..header.property_count {
            let property = CastProperty::load(reader)?;
            if !node.properties.contains_key(&property.name) {
                node.properties.insert(property.name.clone(), property);
            }
        }

        for _ in 0..header.child_count {
            let mut child = CastNode::load(reader)?;
            child.parent_node = Some(node.hash);
            node.child_nodes.push(child);
        }

        Ok(node)
    }

    pub fn clone(&self) -> CastNode {
        Self {
            identifier: self.identifier,
            hash: self.hash,
            properties: self.properties.clone(),
            child_nodes: self.child_nodes.clone(),
            parent_node: self.parent_node,
        }
    }
}

pub struct CastFile {
    pub root_nodes: Vec<CastNode>,
}

impl CastFile {
    pub fn new(capacity: usize) -> Self {
        CastFile {
            root_nodes: Vec::with_capacity(capacity),
        }
    }

    pub fn load<R: Read + Seek>(reader: &mut R) -> Result<Self, std::io::Error> {
        let header: CastHeader = reader.read_struct()?;
        if header.magic != 0x74736163 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid file magic",
            ));
        }
        let mut result = CastFile::new(header.root_nodes as usize);
        for _ in 0..header.root_nodes {
            result.root_nodes.push(CastNode::load(reader)?)
        }
        Ok(result)
    }
}

pub fn load_model_images(model: &Model, file_name: &Path) -> Vec<Option<Image>> {
    let mut img = Vec::new();
    for mats in &model.materials {
        if mats.textures.is_empty() {
            img.push(None);
            continue;
        }
        for images in &mats.textures {
            if images.texture_usage == MaterialTextureRefUsage::Diffuse
                || images.texture_usage == MaterialTextureRefUsage::Albedo
            {
                let directory = file_name.parent().unwrap_or(Path::new("."));
                let f = directory.join(&images.file_name);
                // Assuming `f` is the file name
                let image_file_type = match f.extension() {
                    Some(ext) => match ext.to_str() {
                        Some("png") => ImageFileType::Png,
                        Some("dds") => ImageFileType::Dds,
                        Some("tiff") => ImageFileType::Tiff,
                        _ => {
                            eprintln!("Unsupported file extension: {}", ext.to_string_lossy());
                            ImageFileType::Dds
                        }
                    },
                    None => {
                        eprintln!("File has no extension");
                        ImageFileType::Dds
                    }
                };
                let image = match Image::load(f, image_file_type) {
                    Ok(image) => Some(image),
                    Err(err) => {
                        eprintln!("Failed to load image: {}: {:?}", &images.file_name, err);
                        None
                    }
                };
                img.push(image);
            }
        }
    }
    if img.is_empty() {
        for _ in &model.materials {
            img.push(None);
        }
    }
    img
}

pub fn load_cast_file<R: Read + Seek>(reader: &mut R) -> Option<CastNode> {
    CastFile::load(reader)
        .ok()?
        .root_nodes
        .first()
        .and_then(|cast_model| {
            cast_model
                .child_nodes
                .iter()
                .find(|node| node.identifier == 0x6C646F6D)
                .cloned()
        })
}

pub fn process_model_node(model_node: &CastNode) -> Model {
    let mut model = Model::new();
    model.skeleton = process_skeleton_node(
        model_node
            .child_nodes
            .iter()
            .find(|node| matches!(node.identifier, 0x6C656B73))
            .expect("Skeleton node not found"),
    );
    process_material_nodes(model_node, &mut model);
    process_mesh_nodes(model_node, &mut model);
    model
}

fn process_skeleton_node(skeleton_node: &CastNode) -> Skeleton {
    let mut skeleton = Skeleton::new();
    for bone_node in &skeleton_node.child_nodes {
        if bone_node.identifier == 0x656E6F62 {
            skeleton.bones.push(process_bone_node(bone_node));
        }
    }
    skeleton
}

fn process_bone_node(bone_node: &CastNode) -> Bone {
    Bone {
        name: bone_node
            .properties
            .get("n")
            .and_then(|p| p.values.first().and_then(|v| v.get_string()))
            .map(|s| s.to_string()),
        parent: bone_node
            .properties
            .get("p")
            .and_then(|p| p.values.first().and_then(|v| v.get_int()))
            .unwrap_or(-1),
        local_position: bone_node
            .properties
            .get("lp")
            .and_then(|p| p.values.first().and_then(|v| v.get_vector3())),
        local_rotation: bone_node
            .properties
            .get("lr")
            .and_then(|p| p.values.first().and_then(|v| v.get_quaternion())),
        local_scale: bone_node
            .properties
            .get("s")
            .and_then(|p| p.values.first().and_then(|v| v.get_vector3())),
        world_position: bone_node
            .properties
            .get("wp")
            .and_then(|p| p.values.first().and_then(|v| v.get_vector3())),
        world_rotation: bone_node
            .properties
            .get("wr")
            .and_then(|p| p.values.first().and_then(|v| v.get_quaternion())),
        world_scale: bone_node
            .properties
            .get("s")
            .and_then(|p| p.values.first().and_then(|v| v.get_vector3())),
    }
}

fn process_material_nodes(model_node: &CastNode, model: &mut Model) {
    for child_node in &model_node.child_nodes {
        if child_node.identifier == 0x6C74616D {
            let name = child_node
                .properties
                .get("n")
                .and_then(|p| p.values.first().and_then(|v| v.get_string()))
                .unwrap_or_default()
                .to_string();
            let mut material = Material::new(name);

            if !child_node.child_nodes.is_empty() {
                let albedo_hash = child_node
                    .properties
                    .get("diffuse")
                    .and_then(|p| p.values.first().and_then(|v| v.get_long()))
                    .unwrap_or_else(|| {
                        child_node
                            .properties
                            .get("albedo")
                            .and_then(|p| p.values.first().and_then(|v| v.get_long()))
                            .unwrap_or(0)
                    });

                for texture_node in &child_node.child_nodes {
                    if texture_node.identifier == 0x656C6966 && texture_node.hash == albedo_hash {
                        let file_name = texture_node
                            .properties
                            .get("p")
                            .and_then(|p| p.values.first().and_then(|v| v.get_string()))
                            .unwrap_or_default()
                            .to_string();
                        let texture_ref = MaterialTextureRef {
                            file_name,
                            texture_usage: MaterialTextureRefUsage::Albedo,
                            texture_alias: "".to_string(),
                            texture_hash: texture_node.hash,
                        };
                        material.textures.push(texture_ref);
                        break;
                    }
                }
            }
            model.materials.push(material);
        }
    }
}

fn process_mesh_nodes(model_node: &CastNode, model: &mut Model) {
    let mut mesh_index = 0;
    for child_node in &model_node.child_nodes {
        if child_node.identifier == 0x6873656D {
            // Perform operations on each Mesh node
            let uv_layers = child_node
                .properties
                .get("ul")
                .and_then(|p| p.values.first().and_then(|v| v.get_int()))
                .unwrap_or(0);
            let weight_influence = child_node
                .properties
                .get("mi")
                .and_then(|p| p.values.first().and_then(|v| v.get_int()))
                .unwrap_or(0);

            let mut vertex_buffer = VertexBuffer::builder()
                .colors(0)
                .uv_layers(uv_layers as usize)
                .maximum_influence(weight_influence as usize)
                .build();

            // Vertex Positions
            if let Some(vp_property) = child_node.properties.get("vp") {
                for value in vp_property.values.iter() {
                    if let Some(pos) = value.get_vector3() {
                        vertex_buffer.create().set_position(pos);
                    }
                }
            }
            // Normals
            if let Some(vn_property) = child_node.properties.get("vn") {
                for (i, value) in vn_property.values.iter().enumerate() {
                    if let Some(n) = value.get_vector3() {
                        vertex_buffer.vertex_mut(i).set_normal(n);
                    }
                }
            }
            // UV0
            if let Some(uv_property) = child_node.properties.get("u0") {
                for (i, value) in uv_property.values.iter().enumerate() {
                    if let Some(uv) = value.get_vector2() {
                        vertex_buffer.vertex_mut(i).set_uv(0, uv);
                    }
                }
            }
            // UV1
            if let Some(uv_property) = child_node.properties.get("u1") {
                for (i, value) in uv_property.values.iter().enumerate() {
                    if let Some(uv) = value.get_vector2() {
                        vertex_buffer.vertex_mut(i).set_uv(1, uv);
                    }
                }
            }

            // Faces
            let mut face_buffer = FaceBuffer::new();
            if let Some(property) = child_node.properties.get("f") {
                let values = &property.values;
                for i in (0..values.len()).step_by(3) {
                    let a = values.get(i + 2).and_then(|v| v.get_uint()).unwrap_or(0);
                    let b = values.get(i + 1).and_then(|v| v.get_uint()).unwrap_or(0);
                    let c = values.get(i).and_then(|v| v.get_uint()).unwrap_or(0);
                    face_buffer.push(Face::new(a, b, c));
                }
            }
            let mut mesh = Mesh::new(face_buffer, vertex_buffer);
            mesh.material = Some(mesh_index);
            mesh_index += 1;
            model.meshes.push(mesh);
        }
    }
}
