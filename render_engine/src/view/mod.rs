pub mod camera;
pub mod orbit;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt as _;
use zerocopy::{Immutable, IntoBytes};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
pub struct ViewUniform {
    pub view_proj: Mat4,
    pub camera_world_position: Vec3,
    pub _pad: u32,
}

impl ViewUniform {
    pub const VIEW_UNIFORM_SET: u32 = 0;
    // pub const VIEW_UNIFORM_BINDING: u32 = 0;
    pub const fn bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: Self::VIEW_UNIFORM_SET,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        }
    }
    pub fn add_commands(&self, device: &wgpu::Device, render_pass: &mut wgpu::RenderPass) {
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view_uniform"),
            contents: self.as_bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let camera_bind_group_layout = device.create_bind_group_layout(&Self::bind_group_layout());
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: Self::VIEW_UNIFORM_SET,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        render_pass.set_bind_group(Self::VIEW_UNIFORM_SET, &camera_bind_group, &[]);
    }
}

pub trait CameraView {
    fn to_view_matrix(&self) -> Mat4;
    fn to_camera_uniform(&self) -> ViewUniform;
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_view_uniform_struct_align() {
        let module = naga::front::wgsl::parse_str(include_str!("../shader_common.wgsl")).unwrap();
        crate::verify_wgsl_struct_sized!(ViewUniform, module, view_proj, camera_world_position);
    }
}
