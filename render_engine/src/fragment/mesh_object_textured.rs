use crate::{
    context::Context,
    texture::{CpuTextureInfo, GpuTextureInfo, SampledTexture},
    vertex::mesh_object::MeshObject,
};

// https://github.com/gfx-rs/wgpu/pull/715
// https://github.com/gfx-rs/wgpu/pull/711
// https://github.com/gfx-rs/wgpu/issues/106
// Should work...?
// https://github.com/gfx-rs/wgpu/pull/1995

#[derive(Debug, Clone)]
pub struct MeshObjectTextured {
    /// The context we operate on.
    pub context: Context,
    /// The mesh object we are operating on.
    pub mesh_object: MeshObject,

    /// The textures.
    pub cpu_textures: CpuTextureInfo,
    pub gpu_textures: GpuTextureInfo,
}

impl MeshObjectTextured {
    pub fn new_simple(
        context: Context,
        mesh_object: MeshObject,
        textures: &[wgpu::Texture],
    ) -> Self {
        let device = &context.device;
        let textures: Vec<SampledTexture> = textures
            .iter()
            .cloned()
            .map(|texture| {
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge, // Repeat, MirrorRepeat, ClampToEdge
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear, // Nearest
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    // lod_min_clamp: 0.0,
                    // lod_max_clamp: 5.0,
                    ..Default::default()
                });
                SampledTexture {
                    sampler,
                    texture,
                    texture_type: crate::texture::TextureType::BaseColor,
                }
            })
            .collect();

        let cpu_textures =
            CpuTextureInfo::new(device, &format!("{}", mesh_object.gpu_mesh.name), &textures);
        let gpu_textures = cpu_textures.to_gpu();

        let mut res = Self {
            context,
            mesh_object,
            cpu_textures,
            gpu_textures,
        };
        // Replace the dummy bindgroup with something real.
        res.replace_gpu_data();
        res
    }

    pub fn new(
        context: Context,
        mesh_object: MeshObject,
        sampled_textures: &[SampledTexture],
    ) -> Self {
        let device = &context.device;

        let cpu_textures = CpuTextureInfo::new(
            device,
            &format!("{}", mesh_object.gpu_mesh.name),
            &sampled_textures,
        );
        let gpu_textures = cpu_textures.to_gpu();

        let mut res = Self {
            context,
            mesh_object,
            cpu_textures,
            gpu_textures,
        };
        res.replace_gpu_data();
        res
    }
    pub fn replace_gpu_data(&mut self) {
        self.mesh_object.replace_gpu_data();
        self.gpu_textures = self.cpu_textures.to_gpu();
    }

    pub fn add_commands(&self, render_pass: &mut wgpu::RenderPass) {
        self.gpu_textures.add_commands(render_pass);
        self.mesh_object.add_commands(render_pass);
    }
}
