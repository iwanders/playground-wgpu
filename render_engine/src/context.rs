use std::sync::Arc;
use winit::window::Window;

/// Something that holds the context for rendering.
#[derive(Clone, Debug)]
pub struct Context {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

use crate::target::Target;
pub struct ContextReturn {
    pub target: Target,
    pub context: Context,
}

fn get_necessary_features() -> wgpu::Features {
    // Hmmm... indexing with the vertex into a texture sounds... relevant :D
    // https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Features.html#associatedconstant.STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
    // And they do support push constants.
    // https://docs.rs/wgpu-types/27.0.1/wgpu_types/struct.Features.html#associatedconstant.PUSH_CONSTANTS

    // wgpu::Features::EXPERIMENTAL_PASSTHROUGH_SHADERS // We use this such that we can use slang and spirv shaders.
    // | wgpu::Features::TEXTURE_BINDING_ARRAY // Such that we can bind variable numbers of textures.
    // | wgpu::Features::UNSIZED_BINDING_ARRAY
    // | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
    wgpu::Features::default()
}

impl Context {
    pub async fn new_sized(width: u32, height: u32) -> Result<ContextReturn, crate::Error> {
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
            .await?;
        log::info!("Features: {:#?}", adapter.features());
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: get_necessary_features(),
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },

                required_limits: wgpu::Limits {
                    // This nonsense to allow binding arrays of textures... :<
                    max_binding_array_elements_per_shader_stage: 1024,
                    max_binding_array_sampler_elements_per_shader_stage: 1024,
                    ..Default::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let context = Context { device, queue };
        let target = context.render_surface(width, height);
        Ok(ContextReturn { context, target })
    }

    pub fn render_surface(&self, width: u32, height: u32) -> Target {
        let (texture_format, present_mode, alpha_mode) = (
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::PresentMode::AutoNoVsync,
            wgpu::CompositeAlphaMode::Auto,
        );

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
        Target::new_texture(self.clone(), config)
    }

    pub async fn new_window(window: Arc<Window>) -> Result<ContextReturn, crate::Error> {
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

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: get_necessary_features(),
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
                // we're building for the web we'll have to disable some.
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let (texture_format, present_mode, alpha_mode) = (
            surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]),
            surface_caps.present_modes[0],
            surface_caps.alpha_modes[0],
        );
        let dims = window.inner_size();
        let (width, height) = (dims.width, dims.height);

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
        let context = Context {
            device: device.clone(),
            queue,
        };
        Ok(ContextReturn {
            context: context.clone(),
            target: Target::new_surface(context.clone(), surface, window, config),
        })
    }
}
