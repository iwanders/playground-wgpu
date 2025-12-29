use glam::{Vec3, Vec3A};
use wgpu::util::DeviceExt as _;
use zerocopy::{Immutable, IntoBytes};

// Should we use nice enums, like Directional{direction: Vec3} and then convert it? or just do this?
#[derive(
    Debug, Copy, Clone, PartialEq, Hash, Ord, PartialOrd, Eq, IntoBytes, Immutable, Default,
)]
#[repr(u32)]
pub enum LightType {
    #[default]
    Off = 0,
    Directional = 1, // Directional (rays parallel)
    Omni = 2,        // Spherical light (radiates outward in a circle)
    Ambient = 3,     // Just provides ambient illumination
}

#[derive(Debug, Copy, Clone, PartialEq, IntoBytes, Immutable, Default)]
#[repr(C)]
pub struct Light {
    pub position: Vec3A,
    pub direction: Vec3A,
    pub color: Vec3, // do lights have alpha?
    pub intensity: f32,
    pub light_type: LightType,
    // something something falloff..?
    pub _pad1: [u8; 12],
}
// Tested at the bottom... for wgsl :|

impl Light {
    pub fn omni() -> Self {
        Light {
            light_type: LightType::Omni,
            ..Default::default()
        }
    }
    pub fn directional() -> Self {
        Light {
            light_type: LightType::Directional,
            ..Default::default()
        }
    }
    pub fn with_position<P: Into<Vec3A>>(mut self, position: P) -> Self {
        self.position = position.into();
        self
    }
    pub fn with_direction<P: Into<Vec3A>>(mut self, direction: P) -> Self {
        self.direction = direction.into();
        self
    }
    pub fn with_color<P: Into<Vec3>>(mut self, color: P) -> Self {
        self.color = color.into();
        self
    }
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }
}

// Things that involve rendering on the graphics card.
pub struct CpuLights {
    pub context: crate::Context,
    pub lights: Vec<Light>,
}

impl CpuLights {
    pub const LIGHT_SET: u32 = 1;
    pub const LIGHT_UNIFORM_BINDING: u32 = 0; // <- why is this not used???
    pub fn new(context: crate::Context) -> Self {
        Self {
            context,
            lights: vec![],
        }
    }
    pub fn add_lights(&mut self, lights: &[Light]) {
        self.lights.extend(lights.iter())
    }
    pub fn with_lights(mut self, add_lights: &[Light]) -> Self {
        self.lights.extend(add_lights.iter());
        self
    }

    pub const fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: Self::LIGHT_UNIFORM_BINDING,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("light_bind_group_layout"),
        }
    }

    pub fn to_gpu(&self) -> GpuLights {
        let light_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Light buffer"),
                    contents: self.lights.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let light_bind_group_layout = self
            .context
            .device
            .create_bind_group_layout(&Self::bind_group_layout());

        let light_bind_group = self
            .context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &light_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: Self::LIGHT_UNIFORM_BINDING,
                    resource: light_buffer.as_entire_binding(),
                }],
                label: Some("light_bind_group"),
            });
        GpuLights {
            light_bind_group_layout,
            light_buffer,
            light_bind_group,
        }
    }
}

pub struct GpuLights {
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group: wgpu::BindGroup,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_light_struct_align() {
        let module = naga::front::wgsl::parse_str(include_str!("shader_common.wgsl")).unwrap();
        crate::verify_wgsl_struct_sized!(
            Light, module, position, direction, color, intensity, light_type
        );
    }
}
