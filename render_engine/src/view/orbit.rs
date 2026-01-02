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
        let target_to_camera = self.camera.eye - self.camera.target;

        // Ideally, we'd use the 'up' vector here to determine the rotation.

        // Express the delta into the up frame.
        let camera_rotate_about_axis = vec3(0.0, 0.0, 1.0);
        let rotation = glam::Quat::from_rotation_arc(camera_rotate_about_axis, self.camera.up);

        // Go from left hand to right hand...
        let camera_local_frame = rotation.mul_vec3(target_to_camera);

        let x = camera_local_frame[0];
        let y = camera_local_frame[1];
        let z = camera_local_frame[2];

        // Go to polar coordinates
        let magnitude = target_to_camera.length();
        let mut theta = glam::vec2(x, y).length().atan2(z);
        let mut phi = y.atan2(x);
        let mut rho = magnitude;

        // Perform the changes.
        theta += delta_vertical;
        phi += delta_horizontal;
        rho += delta_distance * target_to_camera.length().max(0.01);

        // Back to cartesian
        let new_x = rho * theta.sin() * phi.cos();
        let new_y = rho * theta.sin() * phi.sin();
        let new_z = rho * theta.cos();

        // And back to global frame.
        let camera_local_frame = rotation.inverse().mul_vec3(vec3(new_x, new_y, new_z));

        // Offset that with the target.
        self.camera.eye = self.camera.target + camera_local_frame;
    }
    pub fn orbit_delta_target(
        &mut self,
        delta_horizontal: f32,
        delta_vertical: f32,
        delta_distance: f32,
    ) {
        // not quite...
        // Move the target location bad on the current orientation and distance to the target. Effectively translating
        // the viewpoint and target with equal values.
        let target_to_camera = self.camera.eye - self.camera.target;
        let distance = target_to_camera.length();

        // Determine the frame of the eye in world coordinates, obtaining its orientation and translation.
        let camera_angle = glam::Quat::from_rotation_arc(vec3(0.0, 0.0, 1.0), target_to_camera);
        let eye_world_frame = Mat4::from_rotation_translation(camera_angle, self.camera.eye);

        // Express change in the local camera frame.
        let new_x = delta_horizontal * distance * 0.1;
        let new_y = delta_vertical * distance * 0.1;
        let new_z = 0.0;

        // Transform the local camera frame changes to the world.
        let camera_local_frame = eye_world_frame.transform_point3(vec3(new_x, new_y, new_z));

        // Subtract the camera eye to determine the delta.
        let delta = camera_local_frame - self.camera.eye;

        // Offset that with the target and the eye.
        self.camera.target += delta;
        self.camera.eye += delta;
    }
}

impl super::CameraView for OrbitCamera {
    fn to_view_matrix(&self) -> Mat4 {
        self.camera.to_view_projection_matrix()
    }

    fn to_camera_uniform(&self) -> super::ViewUniform {
        self.camera.to_camera_uniform()
    }
}
