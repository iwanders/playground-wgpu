use crate::{context::Context, vertex::mesh_object::MeshObject};

// https://github.com/gfx-rs/wgpu/pull/715
// https://github.com/gfx-rs/wgpu/pull/711
// https://github.com/gfx-rs/wgpu/issues/106
// Should work...?
// https://github.com/gfx-rs/wgpu/pull/1995

pub struct SampledTexture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
}

pub struct MeshObjectTextured {
    /// The context we operate on.
    pub context: Context,
    /// The mesh object we are operating on.
    pub mesh_object: MeshObject,
    /// The textures necessary by the fragment shader.
    pub textures: Vec<SampledTexture>,

    /// The texture bind group.
    pub texture_bind_group: wgpu::BindGroup,
}

const fn create_non_zero() -> Option<std::num::NonZero<u32>> {
    unsafe { Some(std::num::NonZero::<u32>::new_unchecked(1)) }
}

impl MeshObjectTextured {
    pub const MESH_OBJECT_TEXTURE_SET: u32 = 3;
    pub const MESH_OBJECT_TEXTURE_BINDING_TEXTURE: u32 = 0;
    pub const MESH_OBJECT_TEXTURE_BINDING_SAMPLER: u32 = 1;

    pub const fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        // Do this clunky bi-directional approach such that the compiler doesn't complain :/
        const FIRST: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
            binding: MeshObjectTextured::MESH_OBJECT_TEXTURE_BINDING_TEXTURE,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: create_non_zero(),
        };
        const SECOND: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
            binding: MeshObjectTextured::MESH_OBJECT_TEXTURE_BINDING_SAMPLER,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: create_non_zero(),
        };
        wgpu::BindGroupLayoutDescriptor {
            entries: &[FIRST, SECOND],
            label: Some("mesh_object_textured_layout"),
        }
    }

    pub fn new(context: Context, mesh_object: MeshObject, textures: &[wgpu::Texture]) -> Self {
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
                SampledTexture { sampler, texture }
            })
            .collect();

        // Create dummy bindgroup.
        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[],
                label: None,
            });
        let texture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[],
                label: None,
            });

        let mut res = Self {
            textures,
            context,
            mesh_object,
            texture_bind_group,
        };
        // Replace the dummy bindgroup with something real.
        res.replace_gpu_data();
        res
    }

    pub fn replace_gpu_data(&mut self) {
        let sampler_pointers: Vec<&wgpu::Sampler> =
            self.textures.iter().map(|z| &z.sampler).collect();
        let texture_views: Vec<wgpu::TextureView> = self
            .textures
            .iter()
            .map(|v| v.texture.create_view(&Default::default()))
            .collect();
        let view_pointers: Vec<&wgpu::TextureView> = texture_views.iter().collect();

        let layout = self
            .context
            .device
            .create_bind_group_layout(&Self::bind_group_layout());
        let texture_bind_group =
            self.context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: Self::MESH_OBJECT_TEXTURE_BINDING_TEXTURE,
                            resource: wgpu::BindingResource::TextureViewArray(&view_pointers),
                        },
                        wgpu::BindGroupEntry {
                            binding: Self::MESH_OBJECT_TEXTURE_BINDING_SAMPLER,
                            resource: wgpu::BindingResource::SamplerArray(&sampler_pointers),
                        },
                    ],
                    label: Some(&format!("{}_bind_group", self.mesh_object.gpu_mesh.name)),
                });
        self.texture_bind_group = texture_bind_group;
    }

    pub fn add_commands(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(Self::MESH_OBJECT_TEXTURE_SET, &self.texture_bind_group, &[]);
        self.mesh_object.add_commands(render_pass);
    }
}
