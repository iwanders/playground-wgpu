pub mod mesh;
// Something that can actually create vertices from the mesh.
pub mod mesh_object;

pub struct VertexCreaterShader {
    pub shader_module: wgpu::ShaderModule,
    pub entry: String,
}

impl VertexCreaterShader {
    pub fn new(shader_module: wgpu::ShaderModule, entry: &str) -> Self {
        Self {
            shader_module,
            entry: entry.to_owned(),
        }
    }
}
