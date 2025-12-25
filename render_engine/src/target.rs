use log::*;
use std::sync::Arc;
use winit::{event::*, event_loop::EventLoop, keyboard::PhysicalKey, window::Window};

use std::path::Path;
#[derive(Debug)]
enum InnerTarget {
    WindowSurface {
        surface: wgpu::Surface<'static>,
        window: Arc<Window>,
    },
    Texture {
        texture: wgpu::Texture,
    },
}
/// Something to render to... a window or an texture?
pub struct Target {
    inner: InnerTarget,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
}

pub struct TargetDestination {
    viewthing: wgpu::TextureView,
    surface_texture: Option<wgpu::SurfaceTexture>,
}
impl TargetDestination {
    pub fn get_view(&self) -> wgpu::TextureView {
        self.viewthing.clone()
    }
    pub fn into_surface(self) -> Option<wgpu::SurfaceTexture> {
        self.surface_texture
    }
    pub fn get_texture_format(&self) -> wgpu::TextureFormat {
        self.get_view().texture().format()
    }
}

impl Target {
    pub fn get_target(&self) -> Option<&wgpu::Surface<'static>> {
        match &self.inner {
            InnerTarget::WindowSurface { surface, window } => Some(surface),
            InnerTarget::Texture { texture } => None,
        }
    }
    pub fn destination(&self) -> Result<TargetDestination, crate::Error> {
        let (viewthing, surface_texture) = match &self.inner {
            InnerTarget::WindowSurface { surface, .. } => {
                let output = surface.get_current_texture()?;
                let viewthing = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let surface_texture = Some(output);

                (viewthing, surface_texture)
            }
            InnerTarget::Texture { texture } => {
                let viewthing = texture.create_view(&wgpu::TextureViewDescriptor::default());
                (viewthing, None)
            }
        };
        Ok(TargetDestination {
            viewthing,
            surface_texture,
        })
    }
    // pub fn get_texture_format(&self) -> Result<wgpu::TextureFormat, crate::Error> {
    //     Ok(match &self.inner {
    //         InnerTarget::WindowSurface { surface, .. } => {
    //             info!("getting surface");
    //             surface.get_current_texture()?.texture.format()
    //         }
    //         InnerTarget::Texture { texture } => texture.format(),
    //     })
    // }

    pub fn reconfigure(&mut self) -> bool {
        match &self.inner {
            InnerTarget::WindowSurface { surface, window } => {
                let dims = window.inner_size();
                if dims.width > 0 && dims.height > 0 {
                    // self.width = width;
                    // self.height = height;
                    //
                    self.config.width = dims.width;
                    self.config.height = dims.height;
                    // self.camera.aspect = self.width as f32 / self.height as f32;

                    surface.configure(&self.device, &self.config);
                    info!("configure happened");
                    true
                } else {
                    false
                }
            }
            InnerTarget::Texture { texture } => false,
        }
    }

    pub fn new_surface(
        device: wgpu::Device,
        surface: wgpu::Surface<'static>,
        window: Arc<Window>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut z = Self {
            inner: InnerTarget::WindowSurface { surface, window },
            config,
            device,
        };
        z.reconfigure();
        z
    }
    pub fn new_texture(device: wgpu::Device, config: wgpu::SurfaceConfiguration) -> Self {
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_desc);
        Self {
            inner: InnerTarget::Texture { texture },
            config,
            device,
        }
    }
    /*
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
    }*/
}
