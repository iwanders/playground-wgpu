use crate::{context::Context, vertex::mesh_object::MeshObject};

pub struct MeshObjectTextured {
    /// The context we operate on.
    pub context: Context,
    /// The mesh object we are operating on.
    pub mesh_object: crate::vertex::mesh_object::MeshObject,
    /// The textures necessary by the fragment shader.
    pub textures: Vec<wgpu::Texture>,
}

impl MeshObjectTextured {
    pub fn new(context: Context, mesh_object: MeshObject, textures: &[wgpu::Texture]) -> Self {
        Self {
            context,
            mesh_object,
            textures: textures.iter().cloned().collect(),
        }
    }
}
