use glam::vec3;
use log::*;
use simple_start::{State, fragment::mesh_object_textured::MeshObjectTextured, view::CameraView};

struct PersistentState {
    mesh_objects_textured: Vec<MeshObjectTextured>,
    depth_format: wgpu::TextureFormat,
    material: Option<simple_start::fragment::PBRMaterial>,
}
struct LocalState {
    persistent: Option<PersistentState>,
}
impl LocalState {
    pub fn new() -> Self {
        Self { persistent: None }
    }
}
impl simple_start::Drawable for LocalState {
    fn initialise(&mut self, state: &mut State) -> Result<(), anyhow::Error> {
        state.camera.camera.eye = vec3(-2.657022, 0.9352254, 1.5044956);

        let gltf_path = std::path::PathBuf::from("../../assets/DamagedHelmet.glb");
        let mesh_objects_textured =
            simple_start::loader::load_gltf_objects(&state.context, &gltf_path)?;

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

        self.persistent = Some(PersistentState {
            mesh_objects_textured,
            material: None,
            depth_format: DEPTH_FORMAT,
        });

        Ok(())
    }
    fn render(&mut self, state: &mut State) -> Result<(), simple_start::Error> {
        state.window.as_ref().map(|k| k.request_redraw());

        // We can't render unless the surface is configured
        if !state.is_surface_configured {
            return Err(wgpu::SurfaceError::Lost.into());
        }

        let device = &state.context.device;
        let persistent = self.persistent.as_mut().unwrap();

        // https://github.com/KhronosGroup/glTF-Sample-Renderer/blob/e6b052db89fb2adbaf31da4565a08265c96c2b9f/source/Renderer/renderer.js#L76-L86
        // Two lights,
        //   a fill light from quat.fromValues(-0.8535534, 0.146446645, -0.353553325, -0.353553444), at intensity 0.5, infinite range.
        //   a directional light from  quat.fromValues(-0.3535534, -0.353553385, -0.146446586, 0.8535534), at intsenity 1.0, infinite range.
        let lights = simple_start::lights::CpuLights::new(state.context.clone()).with_lights(&[
            simple_start::lights::Light::directional()
                .with_direction(
                    glam::Quat::from_array([-0.8535534, 0.146446645, -0.353553325, -0.353553444])
                        * vec3(0.0, 0.0, -1.0),
                )
                .with_intensity(0.5),
            simple_start::lights::Light::directional()
                .with_direction(
                    glam::Quat::from_array([-0.3535534, -0.353553385, -0.146446586, 0.8535534])
                        * vec3(0.0, 0.0, -1.0),
                )
                .with_intensity(1.0),
        ]);
        let gpu_lights = lights.to_gpu();

        let destination = state.target.destination()?;
        let width = destination.width();
        let height = destination.height();
        state.camera.camera.aspect = width as f32 / height as f32;

        let depth_size = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let depth_desc = wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: depth_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: persistent.depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&depth_desc);

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_format = destination.get_texture_format();
        let material = persistent.material.get_or_insert_with(|| {
            let config = simple_start::fragment::PBRMaterialConfig {
                rgba_format: texture_format,
                depth_format: persistent.depth_format,
            };
            info!("Setting up pipeline with {texture_format:?}");

            simple_start::fragment::PBRMaterial::new(
                &state.context,
                &config,
                simple_start::vertex::mesh_object::MeshObject::retrieve_embedded_shader(
                    &state.context.device,
                ),
            )
        });

        let light_bind_group = gpu_lights.light_bind_group;

        let view = destination.get_view();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            // Setup
            render_pass.set_pipeline(&material.render_pipeline);
            // println!("camera: { :?}", state.camera);
            state
                .camera
                .to_camera_uniform()
                .add_commands(device, &mut render_pass);
            render_pass.set_bind_group(
                simple_start::lights::CpuLights::LIGHT_SET,
                &light_bind_group,
                &[],
            );

            // Object properties.
            for obj in persistent.mesh_objects_textured.iter() {
                obj.add_commands(&mut render_pass);
            }
        }

        state.context.queue.submit(Some(encoder.finish()));

        // And copy from the surface to the window canvas.
        if let Some(output) = destination.into_surface() {
            output.present();
        }
        Ok(())
    }
}
async fn async_main() -> std::result::Result<(), anyhow::Error> {
    if true {
        let drawable = LocalState::new();
        simple_start::async_render(drawable, 1024, 768, "/tmp/damaged_helmet_pbr.png").await?;
    }
    let drawable = LocalState::new();
    simple_start::async_main(drawable).await?;

    Ok(())
}

pub fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Info)
        // .filter_level(log::LevelFilter::max())
        .try_init()?;
    pollster::block_on(async_main())?;
    Ok(())
}
