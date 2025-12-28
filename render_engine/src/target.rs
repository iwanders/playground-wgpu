use anyhow::Context as _;
use log::*;
use std::sync::Arc;
use winit::window::Window;

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
    context: crate::Context,
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
    pub fn width(&self) -> u32 {
        self.viewthing.texture().width()
    }
    pub fn height(&self) -> u32 {
        self.viewthing.texture().height()
    }
}

impl Target {
    pub fn get_target(&self) -> Option<&wgpu::Surface<'static>> {
        match &self.inner {
            InnerTarget::WindowSurface { surface, .. } => Some(surface),
            InnerTarget::Texture { .. } => None,
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

                    surface.configure(&self.context.device, &self.config);
                    info!("configure happened");
                    true
                } else {
                    false
                }
            }
            InnerTarget::Texture { .. } => false,
        }
    }

    pub fn new_surface(
        context: crate::Context,
        surface: wgpu::Surface<'static>,
        window: Arc<Window>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let mut z = Self {
            inner: InnerTarget::WindowSurface { surface, window },
            config,
            context,
        };
        z.reconfigure();
        z
    }
    pub fn new_texture(context: crate::Context, config: wgpu::SurfaceConfiguration) -> Self {
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
        let texture = context.device.create_texture(&texture_desc);
        Self {
            inner: InnerTarget::Texture { texture },
            config,
            context,
        }
    }
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::Error> {
        let p: &Path = path.as_ref();

        let width = self.config.width;
        let height = self.config.height;

        // Create a temporary output buffer
        let u32_size = std::mem::size_of::<u32>() as u32;
        let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("capture_buffer"),
            mapped_at_creation: false,
        };
        let buffer = self.context.device.create_buffer(&output_buffer_desc);
        let buffer_slice = buffer.slice(..);

        let texture = match &self.inner {
            InnerTarget::WindowSurface { .. } => todo!(),
            InnerTarget::Texture { texture } => texture,
        };

        // Create commands to copy the current target into the buffer.
        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let extent = wgpu::Extent3d {
            // 2.
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * std::mem::size_of::<u32>() as u32),
                    rows_per_image: Some(width),
                },
            },
            extent,
        );
        self.context.queue.submit(Some(encoder.finish()));

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).unwrap();
        buffer
            .save(p)
            .with_context(|| format!("failed to save to {p:?}"))?;

        Ok(())
    }
}
