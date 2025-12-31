use glam::{Mat4, Vec3};
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// Location of the camera position.
    pub eye: Vec3,

    /// What the camera is looking at, in global coordinates.
    pub target: Vec3,

    /// The up direction of the camera.
    pub up: Vec3,

    /// The visualisation aspect ratio.
    pub aspect: f32,

    /// The vertical field of view.
    pub fovy: f32,

    /// Depth near for the perspective transform.
    pub znear: f32,

    /// depth far for the perspective transform.
    pub zfar: f32,
}
impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, -0.1, 1.3).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            aspect: width as f32 / height as f32,
            fovy: 90.0,
            znear: 0.001,
            zfar: 1000.0,
        }
    }
    pub fn to_view_projection_matrix(&self) -> Mat4 {
        // https://github.com/bitshifter/glam-rs/issues/569
        // Okay, so this doesn't actually do what we need :<
        //let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        // info!("self: {:?}", self);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        // view.col_mut(0)[1] *= -1.0;
        return proj * view;
    }
}
impl super::CameraView for Camera {
    fn to_view_matrix(&self) -> Mat4 {
        self.to_view_projection_matrix()
    }
    fn to_camera_uniform(&self) -> super::ViewUniform {
        {
            let view_proj = self.to_view_matrix();
            let camera_world_position = self.eye.into();
            super::ViewUniform {
                view_proj,
                camera_world_position,
                _pad: Default::default(),
            }
        }
    }
}
