use glam::{Mat4, Vec3, Vec3A, vec3, vec3a};
use log::*;
use simple_start::State;
use wgpu::util::DeviceExt;

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

use zerocopy::{Immutable, IntoBytes};

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
pub struct OurUniform {
    pub view_proj: Mat4,
    pub model_tf: Mat4,
    pub camera_world_position: Vec3A,
}

struct PersistentState {
    shader: wgpu::ShaderModule,
    gpu_mesh: simple_start::mesh::GpuMesh,
    model_tf: Mat4,
}
struct LocalState {
    persistent: Option<PersistentState>,
}
impl LocalState {
    pub fn new() -> Self {
        Self { persistent: None }
    }
}
const USE_SPIRV_SHADER: bool = true;
impl simple_start::Drawable for LocalState {
    fn initialise(&mut self, state: &mut State) -> Result<(), anyhow::Error> {
        state.camera.eye = vec3(-0.6, -0.65, 0.43);
        let device = &state.context.device;

        /*
         * Nope
        Caused by:
          In Device::create_shader_module, label = 'shader.spv'

        Shader 'shader.spv' parsing error: UnsupportedInstruction(Function, CopyLogical)

        */

        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader = if USE_SPIRV_SHADER {
            let config = wgpu::ShaderModuleDescriptorPassthrough {
                label: Some("shader.spv"),
                spirv: Some(wgpu::util::make_spirv_raw(include_bytes!("shader.spv"))),
                entry_point: "".to_owned(),
                // This is unused for SPIR-V
                num_workgroups: (0, 0, 0),
                runtime_checks: wgpu::ShaderRuntimeChecks::unchecked(),
                dxil: None,
                msl: None,
                hlsl: None,
                glsl: None,
                wgsl: None,
            };
            unsafe { device.create_shader_module_passthrough(config) }
        } else {
            device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"))
        };

        // https://github.com/KhronosGroup/glTF-Sample-Assets/tree/a39304cad827573c60d1ae47e4bfbb2ee43d5b13/Models/DragonAttenuation/glTF-Binary
        // let gltf_path = std::path::PathBuf::from("../../assets/DragonDispersion.glb");
        let gltf_path = std::path::PathBuf::from("../../assets/BoxVertexColors.glb");
        let (document, buffers, images) = gltf::import(gltf_path)?;
        // info!("document: {document:#?}");
        let cpu_mesh = simple_start::loader::load_gltf(document, &buffers, 0);

        let gpu_mesh = cpu_mesh.to_gpu(&state.context);

        let model_tf = Mat4::IDENTITY
            * Mat4::from_rotation_x(std::f32::consts::PI)
            * Mat4::from_translation(vec3(0.0, 0.0, 0.5))
            * Mat4::from_scale(Vec3::splat(0.1));

        self.persistent = Some(PersistentState {
            shader,
            gpu_mesh,
            model_tf,
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
        // Something something... fragment shader... set colors? >_<
        let persistent = self.persistent.as_ref().unwrap();
        let shader = &persistent.shader;
        let vertex_buffer = &persistent.gpu_mesh.vertex_buffer;
        let index_buffer = &persistent.gpu_mesh.index_buffer;

        let camera_world_position = state.camera.eye.into();
        let our_uniform = OurUniform {
            view_proj: state.camera.to_view_projection_matrix(),
            camera_world_position,
            model_tf: persistent.model_tf,
        };
        // warn!("our_uniform: {our_uniform:?}");

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Uniform"),
            contents: [our_uniform].as_bytes(),

            usage: wgpu::BufferUsages::STORAGE,
        });

        let l1_theta = simple_start::get_angle_f32(1.2);
        let l2_theta = -simple_start::get_angle_f32(0.7) + 3.14;
        let l1_theta: f32 = 0.3;
        let l2_theta: f32 = 2.3;
        let radius = 0.2;

        let lights = simple_start::lights::CpuLights::new(state.context.clone()).with_lights(&[
            simple_start::lights::Light::directional() // sun left
                .with_direction([1.0, -1.0, 0.5])
                .with_intensity(1.0)
                .with_color([0.1, 0.1, 0.1]),
            simple_start::lights::Light::directional() // sun right
                .with_direction([1.0, 1.0, 1.0])
                .with_intensity(1.0)
                .with_color([0.1, 0.1, 0.1]),
            simple_start::lights::Light::omni() // Orbitter red
                .with_position([l1_theta.cos() * radius, l1_theta.sin() * radius, 0.1])
                .with_intensity(5.0)
                .with_color([1.0, 0.3, 0.3]),
            simple_start::lights::Light::omni() // Orbitter green
                .with_position([l2_theta.cos() * radius, l2_theta.sin() * radius, 0.1])
                .with_intensity(5.0)
                .with_color([0.3, 0.3, 0.1]),
        ]);

        let gpu_lights = lights.to_gpu();

        let destination = state.target.destination()?;
        let width = destination.width();
        let height = destination.height();
        state.camera.aspect = width as f32 / height as f32;

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

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
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&depth_desc);

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let num_indices = persistent.gpu_mesh.index_length;

        // let texture_format = state.target.get_texture_format()?;
        let texture_format = destination.get_texture_format();

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let light_bind_group_layout = gpu_lights.light_bind_group_layout;
        let light_bind_group = gpu_lights.light_bind_group;

        let mesh_group_layout =
            device.create_bind_group_layout(&simple_start::mesh::GpuMesh::MESH_LAYOUT);
        let mesh_group_layout = &mesh_group_layout;

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                    mesh_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: if USE_SPIRV_SHADER {
                    Some("vertexMain")
                } else {
                    None
                },
                // buffers: &[],
                buffers: &[simple_start::mesh::GpuMesh::get_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: if USE_SPIRV_SHADER {
                    Some("fragmentMain")
                } else {
                    None
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            cache: None,
        });

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
            render_pass.push_debug_group("Things");

            // Setup
            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_bind_group(1, &light_bind_group, &[]);

            // Object properties.
            render_pass.set_bind_group(2, &persistent.gpu_mesh.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
            render_pass.pop_debug_group();
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
