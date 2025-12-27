use glam::{Mat4, vec3};

use super::camera::Camera;

#[derive(Copy, Clone, Debug)]
pub struct OrbitCamera {
    pub camera: Camera,
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_up: f32,
    pub amount_down: f32,
}

impl OrbitCamera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            camera: Camera::new(width, height),
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
        }
    }
    pub fn update(&mut self) {
        const CARTESIAN: bool = false;
        if CARTESIAN {
            let s = 0.1;
            self.camera.eye.z += (self.amount_up - self.amount_down) * s;
            self.camera.eye.x += (self.amount_left - self.amount_right) * s;
            self.camera.eye.y += (self.amount_forward - self.amount_backward) * s;
        } else {
            let s = 0.05;
            let delta_vertical = (self.amount_up - self.amount_down) * s;
            let delta_horizontal = -(self.amount_left - self.amount_right) * s;
            let delta_distance = -(self.amount_forward - self.amount_backward) * s;
            self.orbit_delta(delta_horizontal, delta_vertical, delta_distance);
        }
    }
    pub fn orbit_delta(&mut self, delta_horizontal: f32, delta_vertical: f32, delta_distance: f32) {
        // something something, polar coordinates.
        let target_to_camera = self.camera.eye;

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
        self.camera.eye = new_eye;
    }
}

impl super::CameraView for OrbitCamera {
    fn to_view_matrix(&self) -> Mat4 {
        self.camera.to_view_projection_matrix()
    }
}
