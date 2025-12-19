// I started with [this](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/), but that started setting up
// application state while we should be able to just render to an image??
//
// Oh, that is in https://sotrh.github.io/learn-wgpu/showcase/windowless/ so yeah I was on the right track
// with just the adapter, device and queue.
//
// Lets just start with tutorial 2, and pick from tutorial 1 as we see fit.

use anyhow::Context;
use glam::{Mat4, Vec3, vec3};
use log::*;
use std::path::Path;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::{event::*, event_loop::EventLoop, keyboard::PhysicalKey, window::Window};
use zerocopy::{Immutable, IntoBytes};

pub struct State {
    // instance: wgpu::Instance,
    pub surface: Option<wgpu::Surface<'static>>,
    //pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub buffer: wgpu::Buffer,
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    pub window: Option<Arc<Window>>,
    pub camera: Camera,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
}
impl State {
    async fn new_window(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,

            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        Self::new_full(
            size.width,
            size.height,
            Some(adapter),
            Some(window),
            Some(surface),
        )
        .await
    }
    pub async fn new_sized(width: u32, height: u32) -> anyhow::Result<State> {
        Self::new_full(width, height, None, None, None).await
    }
    pub async fn new_full(
        width: u32,
        height: u32,
        adapter: Option<wgpu::Adapter>,
        window: Option<Arc<Window>>,
        surface: Option<wgpu::Surface<'static>>,
    ) -> anyhow::Result<State> {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let adapter = adapter.unwrap_or_else(|| {
            pollster::block_on((async || {
                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::PRIMARY,

                    ..Default::default()
                });

                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: None,
                        force_fallback_adapter: false,
                    })
                    .await
                    .unwrap();
                adapter
            })())
        });

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        // let surface_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        // let present_mode = wgpu::PresentMode::AutoNoVsync;
        // let alpha_mode = wgpu::CompositeAlphaMode::Auto;

        let (texture_format, present_mode, alpha_mode) = surface
            .as_ref()
            .map(|s| {
                let surface_caps = s.get_capabilities(&adapter);
                (
                    surface_caps
                        .formats
                        .iter()
                        .copied()
                        .find(|f| f.is_srgb())
                        .unwrap_or(surface_caps.formats[0]),
                    surface_caps.present_modes[0],
                    surface_caps.alpha_modes[0],
                )
            })
            .unwrap_or((
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::PresentMode::AutoNoVsync,
                wgpu::CompositeAlphaMode::Auto,
            ));
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: width,
            height: height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("capture_buffer"),
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&output_buffer_desc);
        Ok(State {
            surface,
            device,
            queue,
            // instance,
            buffer,
            width,
            height,
            texture,
            texture_view,
            window,
            config,
            camera: Camera::new(width, height),
            is_surface_configured: false,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.config.width = width;
            self.config.height = height;
            self.camera.aspect = self.width as f32 / self.height as f32;
            if let Some(surface) = self.surface.as_ref() {
                surface.configure(&self.device, &self.config);
                self.is_surface_configured = true;
            }
        }
    }
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let p: &Path = path.as_ref();
        let buffer_slice = self.buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, data).unwrap();
        buffer
            .save(p)
            .with_context(|| format!("failed to save to {p:?}"))
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
    fn build_view_projection_matrix(&self) -> Mat4 {
        // https://github.com/bitshifter/glam-rs/issues/569
        // Okay, so this doesn't actually do what we need :<
        //let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        return proj * view;
    }
    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.build_view_projection_matrix(),
        }
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
            info!("up: {:?}, {:?}", self.amount_up, self.amount_down);
            // something something, polar coordinates.
            let target_to_camera = self.eye;
            // Go from left hand to right hand...
            let x = target_to_camera.x;
            let y = target_to_camera.y;
            let z = -target_to_camera.z;
            let magnitude = target_to_camera.length();
            let mut theta = glam::vec2(x, y).length().atan2(z);
            let mut phi = y.atan2(x);
            let rho = magnitude;
            theta += (self.amount_up - self.amount_down) * s;
            phi += (self.amount_left - self.amount_right) * s;

            let new_eye_x = rho * theta.sin() * phi.cos();
            let new_eye_y = rho * theta.sin() * phi.sin();
            let new_eye_z = rho * theta.cos();

            // Don't forget the flip back!
            let new_eye = vec3(new_eye_x, new_eye_y, -new_eye_z);
            info!("old eye; {:?} new eye: {:?}", self.eye, new_eye);
            self.eye = new_eye;
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
    fn render(&mut self, state: &mut State) -> Result<(), wgpu::SurfaceError>;
}

pub struct DummyDraw;
impl Drawable for DummyDraw {
    fn render(&mut self, state: &mut State) -> Result<(), wgpu::SurfaceError> {
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
    pub async fn new_sized(drawable: T, width: u32, height: u32) -> Self {
        Self {
            state: Some(State::new_sized(width, height).await.unwrap()),
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
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        let mut drawable = self.drawable.borrow_mut();
        let drawable = &mut *drawable;

        state.update();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                match drawable.render(state) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        if let Some(window) = state.window.as_ref() {
                            let size = window.inner_size();
                            state.resize(size.width, size.height);
                        }
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match (button, state.is_pressed()) {
                (MouseButton::Left, true) => {}
                (MouseButton::Left, false) => {}
                _ => {}
            },
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
