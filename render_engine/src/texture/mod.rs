use wgpu::{Device, util::DeviceExt as _};
use zerocopy::{Immutable, IntoBytes};

// For all textures... index 0 is unused and denotes not present, this also prevents the situation where the array holds
// nothing.

// Common stuff, from the gltf damaged helmet: metallic roughness, base color, occlusion, normal, emissive
#[derive(Debug, Copy, Clone, PartialEq, IntoBytes, Immutable, Default)]
#[repr(C)]
pub struct TextureUniform {
    pub base_color: u32,
    pub metallic_roughness: u32,
    pub occlusion: u32,
    pub normal: u32,
    pub emissive: u32,
}

impl TextureUniform {
    pub fn create_from_iter<'a, I: Iterator<Item = (usize, &'a SampledTexture)>>(it: I) -> Self {
        let mut base_color = 0;
        let mut metallic_roughness = 0;
        let mut occlusion = 0;
        let mut normal = 0;
        let mut emissive = 0;

        for (index, texture) in it {
            match texture.texture_type {
                TextureType::BaseColor => base_color = index as u32,
                TextureType::MetallicRoughness => metallic_roughness = index as u32,
                TextureType::Occlusion => occlusion = index as u32,
                TextureType::Normal => normal = index as u32,
                TextureType::Emissive => emissive = index as u32,
                _ => {}
            }
        }

        TextureUniform {
            base_color,
            metallic_roughness,
            occlusion,
            normal,
            emissive,
        }
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Hash, Ord, PartialOrd, Eq, IntoBytes, Immutable, Default,
)]
#[repr(u32)]
pub enum TextureType {
    #[default]
    None = 0,
    BaseColor = 1,
    MetallicRoughness = 2,
    Occlusion = 3,
    Normal = 4,
    Emissive = 5,
}

#[derive(Clone, Debug)]
pub struct SampledTexture {
    pub sampler: wgpu::Sampler,
    pub texture: wgpu::Texture,
    pub texture_type: TextureType,
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
        let dummy = Some(SampledTexture {
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear, // Nearest
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            texture: device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 1, // does this work? lol
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("dummy_0th_texture"),
                view_formats: &[],
            }),
            texture_type: TextureType::None,
        });

        Self {
            device: device.clone(),
            textures: dummy.iter().chain(textures.iter()).cloned().collect(),
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

        let texture_uniform = TextureUniform::create_from_iter(self.textures.iter().enumerate());

        let texture_uniform_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_texture_uniform", self.name)),
                    contents: texture_uniform.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });
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
                wgpu::BindGroupEntry {
                    binding: GpuTextureInfo::TEXTURE_BINDING_UNIFORM_META,
                    resource: texture_uniform_buffer.as_entire_binding(),
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
    pub const TEXTURE_BINDING_UNIFORM_META: u32 = 2;

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
        const THIRD: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
            binding: GpuTextureInfo::TEXTURE_BINDING_UNIFORM_META,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        wgpu::BindGroupLayoutDescriptor {
            entries: &[FIRST, SECOND, THIRD],
            label: Some("mesh_object_textured_layout"),
        }
    }
    pub fn add_commands(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(Self::TEXTURE_SET, &self.bind_group, &[]);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_texture_struct_align() {
        let module = naga::front::wgsl::parse_str(include_str!("../shader_common.wgsl")).unwrap();
        crate::verify_wgsl_struct_sized!(
            TextureUniform,
            module,
            base_color,
            metallic_roughness,
            occlusion,
            normal,
            emissive
        );
    }
}
