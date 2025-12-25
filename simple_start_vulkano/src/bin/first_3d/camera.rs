use glam::{Mat4, Vec3, vec3, vec4};
use log::*;
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_up: f32,
    pub amount_down: f32,
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
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
        }
    }

    pub fn to_view_projection_matrix(&self) -> Mat4 {
        // https://github.com/bitshifter/glam-rs/issues/569
        // Okay, so this doesn't actually do what we need :<
        //let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        // info!("self: {:?}", self);
        let mut proj =
            Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        // info!("proj: {:?}", proj);
        // info!("proj: {:#?}", proj);
        proj.y_axis.y *= -1.0;
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        return proj * view;
    }

    pub fn orbit_delta(&mut self, delta_horizontal: f32, delta_vertical: f32, delta_distance: f32) {
        // something something, polar coordinates.
        let target_to_camera = self.eye;

        // Go from left hand to right hand...
        let x = target_to_camera.x;
        let y = target_to_camera.y;
        let z = -target_to_camera.z;
        // Go to polar coordinates
        let magnitude = target_to_camera.length();
        let mut theta = glam::vec2(x, y).length().atan2(z);
        let mut phi = y.atan2(x);
        let mut rho = magnitude;

        // Perform the changes.
        theta += delta_vertical;
        phi += delta_horizontal;
        rho += delta_distance;

        // Back to cartesian
        let new_eye_x = rho * theta.sin() * phi.cos();
        let new_eye_y = rho * theta.sin() * phi.sin();
        let new_eye_z = rho * theta.cos();

        // Don't forget the flip back between the left and rh coordinate frames.
        let new_eye = vec3(new_eye_x, new_eye_y, -new_eye_z);
        self.eye = new_eye;
    }
    pub fn update(&mut self) {
        const CARTESIAN: bool = false;
        if CARTESIAN {
            let s = 0.1;
            self.eye.z += (self.amount_up - self.amount_down) * s;
            self.eye.x += (self.amount_left - self.amount_right) * s;
            self.eye.y += (self.amount_forward - self.amount_backward) * s;
        } else {
            let s = 0.05;
            let delta_vertical = (self.amount_up - self.amount_down) * s;
            let delta_horizontal = -(self.amount_left - self.amount_right) * s;
            let delta_distance = -(self.amount_forward - self.amount_backward) * s;
            self.orbit_delta(delta_horizontal, delta_vertical, delta_distance);
        }
    }
}
