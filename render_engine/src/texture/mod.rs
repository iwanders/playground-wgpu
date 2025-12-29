use wgpu::Device;

#[derive(Clone, Debug)]
pub struct SampledTexture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
}
#[derive(Clone, Debug)]
pub struct CpuTextureInfo {
    pub device: wgpu::Device,
    pub name: String,
    pub textures: Vec<SampledTexture>,
}

const fn create_non_zero() -> Option<std::num::NonZero<u32>> {
    unsafe { Some(std::num::NonZero::<u32>::new_unchecked(512)) }
}

impl CpuTextureInfo {
    pub fn new(device: &Device, name: &str, textures: &[SampledTexture]) -> Self {
        Self {
            device: device.clone(),
            textures: textures.to_vec(),
            name: name.to_string(),
        }
    }

    pub fn to_gpu(&self) -> GpuTextureInfo {
        let sampler_pointers: Vec<&wgpu::Sampler> =
            self.textures.iter().map(|z| &z.sampler).collect();
        let texture_views: Vec<wgpu::TextureView> = self
            .textures
            .iter()
            .map(|v| v.texture.create_view(&Default::default()))
            .collect();
        let view_pointers: Vec<&wgpu::TextureView> = texture_views.iter().collect();

        let layout = self
            .device
            .create_bind_group_layout(&GpuTextureInfo::bind_group_layout());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: GpuTextureInfo::TEXTURE_BINDING_TEXTURE,
                    resource: wgpu::BindingResource::TextureViewArray(&view_pointers),
                },
                wgpu::BindGroupEntry {
                    binding: GpuTextureInfo::TEXTURE_BINDING_SAMPLER,
                    resource: wgpu::BindingResource::SamplerArray(&sampler_pointers),
                },
            ],
            label: Some(&self.name),
        });
        GpuTextureInfo { bind_group }
    }
}

pub struct GpuTextureInfo {
    pub bind_group: wgpu::BindGroup,
}

impl GpuTextureInfo {
    pub const TEXTURE_SET: u32 = 3;
    pub const TEXTURE_BINDING_TEXTURE: u32 = 0;
    pub const TEXTURE_BINDING_SAMPLER: u32 = 1;

    pub const fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        // Do this clunky bi-directional approach such that the compiler doesn't complain :/
        const FIRST: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
            binding: GpuTextureInfo::TEXTURE_BINDING_TEXTURE,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: create_non_zero(),
        };
        const SECOND: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
            binding: GpuTextureInfo::TEXTURE_BINDING_SAMPLER,
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
    pub fn add_commands(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(Self::TEXTURE_SET, &self.bind_group, &[]);
    }
}
