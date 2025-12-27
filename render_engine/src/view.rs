use glam::{Mat4, Vec3, Vec3A};
use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
pub struct OurUniform {
    pub view_proj: Mat4,
    pub model_tf: Mat4,
    pub camera_world_position: Vec3A,
}
