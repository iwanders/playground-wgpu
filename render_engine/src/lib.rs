// I started with [this](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/), but that started setting up
// application state while we should be able to just render to an image??
//
// Oh, that is in https://sotrh.github.io/learn-wgpu/showcase/windowless/ so yeah I was on the right track
// with just the adapter, device and queue.
//
// Lets just start with tutorial 2, and pick from tutorial 1 as we see fit.

use anyhow::Context as WithContext;
use glam::{Mat4, Vec3, vec3};
use log::*;
use std::path::Path;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::{event::*, event_loop::EventLoop, keyboard::PhysicalKey, window::Window};
use zerocopy::{Immutable, IntoBytes};

pub mod context;
pub mod mesh;
pub mod scene;
pub mod target;
pub mod view;
pub mod visualiser;

use context::{Context, ContextReturn};
use target::Target;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to obtain adapter: {0:?}")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    #[error("failed to obtain device: {0:?}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("failed to obtain surface: {0:?}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}

pub struct State {
    pub context: context::Context,
    pub target: target::Target,
    // instance: wgpu::Instance,
    // pub surface: Option<wgpu::Surface<'static>>,
    //pub surface: wgpu::Surface<'static>,
    // pub device: wgpu::Device,
    // pub queue: wgpu::Queue,
    // pub buffer: wgpu::Buffer,
    // pub texture: wgpu::Texture,
    // pub texture_view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub window: Option<Arc<Window>>,
    pub camera: Camera,
    // pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub mouse_left_down: bool,
    pub mouse_position: winit::dpi::PhysicalPosition<f64>,
}
impl State {
    async fn new_window(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();
        Self::new_full(size.width, size.height, Some(window)).await
    }
    pub async fn new_sized(width: u32, height: u32) -> anyhow::Result<State> {
        Self::new_full(width, height, None).await
    }
    pub async fn new_full(
        width: u32,
        height: u32,
        window: Option<Arc<Window>>,
    ) -> anyhow::Result<State> {
        let z: context::ContextReturn = if let Some(window) = window.clone() {
            context::Context::new_window(window).await
        } else {
            context::Context::new_sized(width, height).await
        }?;
        let context::ContextReturn { context, target } = z;

        Ok(State {
            context,
            target,
            // instance,
            // buffer,
            width,
            height,
            // texture,
            // texture_view,
            window,
            // config,
            camera: Camera::new(width, height),
            is_surface_configured: false,
            mouse_left_down: false,
            mouse_position: Default::default(),
        })
    }

    pub async fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn update(&mut self) {
        self.camera.update();
    }

    pub fn handle_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        let amount = if pressed { 1.0 } else { 0.0 };
        match key {
            KeyCode::KeyW => {
                self.camera.amount_forward = amount;
                true
            }
            KeyCode::KeyS => {
                self.camera.amount_backward = amount;
                true
            }
            KeyCode::KeyA => {
                self.camera.amount_left = amount;
                true
            }
            KeyCode::KeyD => {
                self.camera.amount_right = amount;
                true
            }
            KeyCode::KeyE => {
                self.camera.amount_up = amount;
                true
            }
            KeyCode::KeyQ => {
                self.camera.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn add_screenshot_to_encoder(
        &self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub fn get_current_time_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    return duration_since_epoch.as_secs_f64();
}

pub fn get_angle_f32(rate: f32) -> f32 {
    (crate::get_current_time_f64() * rate as f64).rem_euclid(2.0 * std::f64::consts::PI) as f32
}

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
        info!("self: {:?}", self);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        return proj * view;
    }
    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.to_view_projection_matrix(),
        }
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
// That means that in normalized device coordinates (opens new window), the x-axis and y-axis are in the range of -1.0 to +1.0, and the z-axis is 0.0 to +1.0.

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
pub struct CameraUniform {
    // ~We can't use cgmath with bytemuck directly, so we'll have~
    // we use glam so we can.
    pub view_proj: Mat4,
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
        }
    }
}

use std::sync::Arc;

pub trait Drawable {
    fn render(&mut self, state: &mut State) -> Result<(), crate::Error>;
    fn initialise(&mut self, state: &mut State) -> Result<(), anyhow::Error>;
}

pub struct DummyDraw;
impl Drawable for DummyDraw {
    fn render(&mut self, state: &mut State) -> Result<(), crate::Error> {
        let _ = state;
        Ok(())
    }
    fn initialise(&mut self, state: &mut State) -> Result<(), anyhow::Error> {
        let _ = state;
        Ok(())
    }
}

pub struct App<T: Drawable> {
    pub state: Option<State>,
    pub drawable: std::cell::RefCell<T>,
}

impl<T: Drawable> App<T> {
    pub fn new(drawable: T) -> Self {
        Self {
            state: None,
            drawable: drawable.into(),
        }
    }
    pub async fn new_sized(mut drawable: T, width: u32, height: u32) -> Self {
        let mut state = State::new_sized(width, height).await.unwrap();
        drawable.initialise(&mut state).unwrap();
        Self {
            state: Some(state),
            drawable: drawable.into(),
        }
    }
    pub async fn render_to_surface(&mut self) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        let mut drawable = self.drawable.borrow_mut();
        let drawable = &mut *drawable;
        state.is_surface_configured = true;
        drawable.render(state).unwrap()
    }
}

impl<T: Drawable> winit::application::ApplicationHandler<State> for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(State::new_window(window)).unwrap());
        self.drawable
            .borrow_mut()
            .initialise(self.state.as_mut().unwrap())
            .with_context(|| "initialise failed")
            .unwrap()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let _ = window_id;
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        let mut drawable = self.drawable.borrow_mut();
        let drawable = &mut *drawable;

        state.update();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                error!("Resize happened");
                state.is_surface_configured = state.target.reconfigure();
            }
            WindowEvent::RedrawRequested => {
                match drawable.render(state) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(
                        Error::SurfaceError(wgpu::SurfaceError::Lost)
                        | Error::SurfaceError(wgpu::SurfaceError::Outdated),
                    ) => {
                        if let Some(window) = state.window.as_ref() {
                            let ContextReturn { context, target } =
                                pollster::block_on(Context::new_window(window.clone())).unwrap();
                            state.context = context;
                            state.target = target;
                            // state.is_surface_configured = state.target.reconfigure();
                            state.is_surface_configured = false;
                        }
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => match (button, button_state.is_pressed()) {
                (MouseButton::Left, v) => state.mouse_left_down = v,
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                if state.mouse_left_down {
                    let s = (std::f32::consts::PI / 1920.0) * 2.0;
                    let dx = (position.x - state.mouse_position.x) as f32 * s;
                    let dy = (position.y - state.mouse_position.y) as f32 * s;
                    state.camera.orbit_delta(-dx, dy, 0.0);
                }
                state.mouse_position = position;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let s = -0.1;
                if let winit::event::MouseScrollDelta::LineDelta(_horizontal, vertical) = delta {
                    state.camera.orbit_delta(0.0, 0.0, s * vertical);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                let _ = state.handle_key(code, key_state.is_pressed());
            }
            _ => {}
        }
    }
}

pub async fn async_main(drawable: impl Drawable) -> std::result::Result<(), anyhow::Error> {
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(drawable);
    event_loop.run_app(&mut app)?;
    Ok(())
}
pub async fn async_render<P: AsRef<Path>>(
    drawable: impl Drawable,
    width: u32,
    height: u32,
    path: P,
) -> std::result::Result<(), anyhow::Error> {
    let p: &Path = path.as_ref();
    let mut app = App::new_sized(drawable, width, height).await;
    app.render_to_surface().await;
    if let Some(state) = app.state.as_ref() {
        state.save(path).await?;
    }

    Ok(())
}

pub fn run(drawable: impl Drawable) -> anyhow::Result<()> {
    env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Warn)
        // .filter_level(log::LevelFilter::max())
        .try_init()?;
    pollster::block_on(async_main(drawable))?;

    Ok(())
}
