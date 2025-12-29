use glam::{Affine3A, Mat4, Vec3, vec3};
use log::*;
use simple_start::{State, fragment::mesh_object_textured::MeshObjectTextured, view::CameraView};

use gltf;

// How do we ehm, pull this hot mess apart?
// Lights were easy
// Is each material just a different pipeline?
//  A material may require multiple passess, consider glowing surfaces?
//  Lets just start simple.
//  How do we handle vertices with optionals? Like UV map, and how do we handle optional textures?
//  VertexAttribute is 'fixed'... Can we just expose an index in there and then trampoline into some buffer that
//  is optional (or zero length?)
//
// Only instanced meshes, if you have a single mesh the instance count is just zero.
//
//
//

use simple_start::vertex::mesh_object::MeshObject;
struct PersistentState {
    mesh_object: MeshObject,
    mesh_objects_textured: Vec<MeshObjectTextured>,
    depth_format: wgpu::TextureFormat,
    material: Option<simple_start::fragment::PhongLikeMaterial>,
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
        state.camera.camera.eye = vec3(-1.368807, 2.1078022, 0.92118156);

        // https://github.com/KhronosGroup/glTF-Sample-Assets/tree/a39304cad827573c60d1ae47e4bfbb2ee43d5b13/Models/DragonAttenuation/glTF-Binary
        // let gltf_path = std::path::PathBuf::from("../../assets/DragonDispersion.glb");
        let gltf_path = std::path::PathBuf::from("../../assets/DamagedHelmet.glb");
        // let gltf_path = std::path::PathBuf::from("../../assets/BoxVertexColors.glb");

        // let gltf_path = std::path::PathBuf::from("../../assets/mailbox_self/mailbox.glb"); // With a texture!

        let mesh_objects_textured =
            simple_start::loader::load_gltf_objects(&state.context, &gltf_path)?;

        let (document, buffers, images) = gltf::import(gltf_path)?;
        // info!("document: {document:#?}");
        let textures: Vec<wgpu::Texture> = images
            .iter()
            .map(|z| simple_start::loader::load_gltf_texture(&state.context, z))
            .collect();
        let cpu_mesh = simple_start::loader::load_gltf(&document, &buffers, 0);

        let poly_count_per_mesh = cpu_mesh.index.len() / 3;

        let gpu_mesh = cpu_mesh.to_gpu(&state.context);

        let mut mesh_object =
            simple_start::vertex::mesh_object::MeshObject::new(state.context.clone(), gpu_mesh);
        mesh_object.set_single_transform(&Mat4::from_scale(Vec3::splat(0.2)));
        // mesh_object.set_transforms(&[Mat4::IDENTITY, Mat4::from_translation(vec3(1.5, 0.0, 0.0))]);
        let mut many_transforms = vec![];
        for x in 0..100 {
            for y in 0..100 {
                for z in 0..100 {
                    let value = Mat4::from_translation(vec3(
                        x as f32 * 1.5,
                        -y as f32 * 1.5,
                        -z as f32 * 1.5,
                    )) * Mat4::from_scale(Vec3::splat(0.2))
                        * Mat4::from_rotation_x(3.14 / 2.0);
                    many_transforms.push(value);
                }
            }
        }
        // mesh_object.set_transforms(&many_transforms);
        // info!(
        //     "total objects: {}, each has {} polygons, for a total of {}",
        //     many_transforms.len(),
        //     poly_count_per_mesh,
        //     many_transforms.len() * poly_count_per_mesh
        // );

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
        mesh_object.replace_gpu_data();

        // let mesh_object_textured =
        //     MeshObjectTextured::new_simple(state.context.clone(), mesh_object.clone(), &textures);

        self.persistent = Some(PersistentState {
            mesh_object,
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

        let l1_theta: f32 = 0.3;
        let l2_theta: f32 = 2.3;
        let radius = 0.2;

        let lights = simple_start::lights::CpuLights::new(state.context.clone()).with_lights(&[
            simple_start::lights::Light::directional() // sun left
                .with_direction([-1.0, -1.0, 0.5])
                .with_intensity(0.2)
                .with_color([0.1, 0.1, 0.1]),
            simple_start::lights::Light::directional() // sun right
                .with_direction([1.0, 1.0, 1.0])
                .with_intensity(1.0)
                .with_color([0.1, 0.1, 0.1]),
            simple_start::lights::Light::omni() // Orbitter red
                .with_position([l1_theta.cos() * radius, l1_theta.sin() * radius, 0.1])
                .with_intensity(5.0)
                .with_color([2.0, 1.3, 0.3]),
            simple_start::lights::Light::omni() // Orbitter green
                .with_position([l2_theta.cos() * radius, l2_theta.sin() * radius, 0.1])
                .with_intensity(5.0)
                .with_color([1.3, 1.3, 0.1]),
        ]);

        let gpu_lights = lights.to_gpu();

        let destination = state.target.destination()?;
        let width = destination.width();
        let height = destination.height();
        state.camera.camera.aspect = width as f32 / height as f32;

        let depth_size = wgpu::Extent3d {
            // 2.
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

        // let num_indices = persistent.gpu_mesh.index_length;

        // let texture_format = state.target.get_texture_format()?;
        let texture_format = destination.get_texture_format();

        // pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb; // 1.
        // pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

        let material = persistent.material.get_or_insert_with(|| {
            let config = simple_start::fragment::PhongLikeMaterialConfig {
                rgba_format: texture_format,
                depth_format: persistent.depth_format,
            };
            info!("Setting up pipeline with {texture_format:?}");

            simple_start::fragment::PhongLikeMaterial::new(
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
                            g: 0.2,
                            b: 0.3,
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
            println!("camera: { :?}", state.camera);
            state
                .camera
                .to_camera_uniform()
                .add_commands(device, &mut render_pass);
            // .render_pass
            // .set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(
                simple_start::lights::CpuLights::LIGHT_SET,
                &light_bind_group,
                &[],
            );

            // Object properties.
            // persistent.mesh_object.add_commands(&mut render_pass);
            for obj in persistent.mesh_objects_textured.iter() {
                obj.add_commands(&mut render_pass);
            }
            // render_pass.set_bind_group(2, &persistent.gpu_mesh.bind_group, &[]);
            // render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            // render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // render_pass.draw_indexed(0..num_indices, 0, 0..1);
            // render_pass.pop_debug_group();
        }

        // state
        //     .add_screenshot_to_encoder(&mut encoder)
        //     .with_context(|| "adding screenshot to encoder failed")
        //     .unwrap();
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
        simple_start::async_render(drawable, 1024, 768, "/tmp/first_3d.png").await?;
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
    println!("Hello, world! ");
    Ok(())
}
