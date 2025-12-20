use anyhow::Context;
use glam::{Mat4, Vec3, Vec3A, vec3};
use log::*;
use simple_start::State;
use wgpu::util::DeviceExt;

use gltf;
// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/
// Skipping over textures and bindgroups
// https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/

// https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/3d-meshes/a-simple-example.html

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
pub struct OurUniform {
    // ~We can't use cgmath with bytemuck directly, so we'll have~
    // we use glam so we can.
    pub view_proj: Mat4,
    pub model_tf: Mat4,
    pub camera_world_position: Vec3A,
}

use zerocopy::{Immutable, IntoBytes};
#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable, Default)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    color: Vec3,
}
impl Vertex {
    pub const fn pnc(position: Vec3, normal: Vec3, color: Vec3) -> Self {
        Self {
            position,
            normal,
            color,
        }
    }
}

// Attrib has to be in sync with Vertex.
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3,  1 => Float32x3,  2 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

fn load_gltf(
    document: gltf::Document,
    buffers: &[gltf::buffer::Data],
    desired_index: usize,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertex_buffer = Vec::<Vertex>::new();
    let mut index_buffer: Vec<u32> = Vec::new();
    let mut found_indices = false;
    for scene in document.scenes() {
        for (node_index, node) in scene.nodes().enumerate() {
            if node_index != desired_index {
                continue;
            }
            if let Some(mesh) = node.mesh() {
                for (mesh_index, primitive) in mesh.primitives().enumerate() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                    info!("new primitive");

                    // Access vertex positions
                    if let Some(positions) = reader.read_positions() {
                        for p in positions {
                            vertex_buffer.push(Vertex::default());
                            let vertex = vertex_buffer.last_mut().unwrap();
                            // Do something with the position [p[0], p[1], p[2]]
                            // println!("Position: {:?}", p);
                            vertex.position = vec3(p[0], p[1], p[2]);
                            vertex.color = vec3(p[0], p[1], p[2]);
                        }
                    }

                    // Access normals
                    if let Some(normals) = reader.read_normals() {
                        for (ni, n) in normals.enumerate() {
                            // Do something with the normal [n[0], n[1], n[2]]
                            vertex_buffer[ni].normal = vec3(n[0], n[1], n[2]);
                        }
                    }
                    // Access indices
                    if let Some(indices) = reader.read_indices() {
                        match indices {
                            ::gltf::mesh::util::ReadIndices::U8(iter) => {
                                index_buffer.extend(iter.map(|v| v as u32));
                            }
                            ::gltf::mesh::util::ReadIndices::U16(iter) => {
                                index_buffer.extend(iter.map(|v| v as u32));
                            }
                            ::gltf::mesh::util::ReadIndices::U32(iter) => {
                                index_buffer.extend(iter);
                            }
                        }
                    }
                    // Access texture coordinates (TexCoords)
                    if let Some(tex_coords) = reader.read_tex_coords(0) {
                        for tc in tex_coords.into_f32() {
                            // Do something with the texture coord [tc[0], tc[1]]
                        }
                    }
                }
            }
        }
    }
    (vertex_buffer, index_buffer)
}

struct PersistentState {
    shader: wgpu::ShaderModule,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    model_tf: Mat4,
    index_length: u32,
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
        state.camera.eye = vec3(-0.6, -0.65, 0.43);
        let device = &state.device;
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // https://github.com/KhronosGroup/glTF-Sample-Assets/tree/a39304cad827573c60d1ae47e4bfbb2ee43d5b13/Models/DragonAttenuation/glTF-Binary
        let gltf_path = std::path::PathBuf::from("../../assets/DragonDispersion.glb");
        let (document, buffers, images) = gltf::import(gltf_path)?;
        info!("document: {document:#?}");
        let (mut vertices, indices) = load_gltf(document, &buffers, 0);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices.as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_length = indices.len() as u32;
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: indices.as_bytes(),
            usage: wgpu::BufferUsages::INDEX,
        });

        let model_tf = Mat4::IDENTITY
            * Mat4::from_rotation_x(std::f32::consts::PI)
            * Mat4::from_translation(vec3(0.0, 0.0, 0.5))
            * Mat4::from_scale(Vec3::splat(0.1));

        self.persistent = Some(PersistentState {
            shader,
            vertex_buffer,
            index_buffer,
            index_length,
            model_tf,
        });

        Ok(())
    }
    fn render(&mut self, state: &mut State) -> Result<(), wgpu::SurfaceError> {
        state.window.as_ref().map(|k| k.request_redraw());

        // We can't render unless the surface is configured
        if !state.is_surface_configured {
            return Err(wgpu::SurfaceError::Lost);
        }

        let device = &state.device;
        // Something something... fragment shader... set colors? >_<
        let persistent = self.persistent.as_ref().unwrap();
        let shader = &persistent.shader;
        let vertex_buffer = &persistent.vertex_buffer;
        let index_buffer = &persistent.index_buffer;

        let camera_world_position = state.camera.eye.into();
        let our_uniform = OurUniform {
            view_proj: state.camera.to_view_projection_matrix(),
            camera_world_position,
            model_tf: persistent.model_tf,
        };
        warn!("our_uniform: {our_uniform:?}");

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Uniform"),
            contents: [our_uniform].as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

        let depth_size = wgpu::Extent3d {
            // 2.
            width: state.width.max(1),
            height: state.height.max(1),
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

        let num_indices = persistent.index_length;

        let texture_format = state.texture_view.texture().format();

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: None,
                // buffers: &[],
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: None,
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
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
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

        #[allow(unused_assignments)]
        let mut viewthing = None;
        let mut surface_texture = None;
        let view = if let Some(surface) = state.surface.as_ref() {
            let output = surface.get_current_texture()?;
            viewthing = Some(
                output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
            surface_texture = Some(output);

            viewthing.as_ref().unwrap()
        } else {
            info!("Not surface");
            &state.texture_view
        };

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
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

            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        info!("running queue");
        state
            .add_screenshot_to_encoder(&mut encoder)
            .with_context(|| "adding screenshot to encoder failed")
            .unwrap();
        state.queue.submit(Some(encoder.finish()));

        // And copy from the surface to the window canvas.
        if let Some(output) = surface_texture {
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
