pub mod camera;
pub mod orbit;
use glam::{Mat4, Vec3A};
use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
pub struct ViewUniform {
    pub view_proj: Mat4,
    pub camera_world_position: Vec3A,
}

pub trait CameraView {
    fn to_view_matrix(&self) -> Mat4;
    fn to_camera_uniform(&self) -> ViewUniform {
        let view_proj = self.to_view_matrix();
        let camera_world_position = view_proj.col(3).truncate().into();
        ViewUniform {
            view_proj,
            camera_world_position,
        }
    }
}
