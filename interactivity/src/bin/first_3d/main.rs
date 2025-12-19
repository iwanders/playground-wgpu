use anyhow::Context;
use glam::{Mat4, Vec3, vec3};
use log::*;
use simple_start::State;
use wgpu::util::DeviceExt;

// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/
// Skipping over textures and bindgroups
// https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/

// https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/3d-meshes/a-simple-example.html

use zerocopy::{Immutable, IntoBytes};
#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
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

const VERTICES: [Vertex; 16] = [
    // The base
    Vertex::pnc(
        vec3(-0.5, -0.5, -0.3),
        vec3(0.0, -1.0, 0.0),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.5, -0.5, -0.3),
        vec3(0.0, -1.0, 0.0),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.5, 0.5, -0.3),
        vec3(0.0, -1.0, 0.0),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(-0.5, 0.5, -0.3),
        vec3(0.0, -1.0, 0.0),
        vec3(1.0, 1.0, 1.0),
    ),
    // Face sides have their own copy of the vertices
    // because they have a different normal vector.
    Vertex::pnc(
        vec3(-0.5, -0.5, -0.3),
        vec3(0.0, -0.848, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.5, -0.5, -0.3),
        vec3(0.0, -0.848, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.0, 0.0, 0.5),
        vec3(0.0, -0.848, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.5, -0.5, -0.3),
        vec3(0.848, 0.0, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.5, 0.5, -0.3),
        vec3(0.848, 0.0, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.0, 0.0, 0.5),
        vec3(0.848, 0.0, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.5, 0.5, -0.3),
        vec3(0.0, 0.848, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(-0.5, 0.5, -0.3),
        vec3(0.0, 0.848, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.0, 0.0, 0.5),
        vec3(0.0, 0.848, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(-0.5, 0.5, -0.3),
        vec3(-0.848, 0.0, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(-0.5, -0.5, -0.3),
        vec3(-0.848, 0.0, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
    Vertex::pnc(
        vec3(0.0, 0.0, 0.5),
        vec3(-0.848, 0.0, 0.53),
        vec3(1.0, 1.0, 1.0),
    ),
];
const INDICES: &[u16] = &[
    // Base
    0, 1, 2, //
    0, 2, 3, //
    // side
    4, 5, 6, //
    7, 8, 9, //
    10, 11, 12, //
    13, 14, 15,
];

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

struct PersistentState {
    shader: wgpu::ShaderModule,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
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
        state.camera.eye = vec3(0.6, -0.71, 0.904);
        let device = &state.device;
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let mut vs = VERTICES;
        for x in vs.iter_mut() {
            // let angle = simple_start::get_angle_f32(0.2);
            let angle = 0.6;
            x.position = Mat4::from_rotation_z(angle).transform_point3(x.position);
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vs.as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: INDICES.as_bytes(),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.persistent = Some(PersistentState {
            shader,
            vertex_buffer,
            index_buffer,
        });
        Ok(())
    }
    fn render(&mut self, state: &mut State) -> Result<(), wgpu::SurfaceError> {
        state.window.as_ref().map(|k| k.request_redraw());

        // We can't render unless the surface is configured
        if !state.is_surface_configured {
            return Err(wgpu::SurfaceError::Lost);
        }
        let camera_uniform = state.camera.to_uniform();
        warn!("camera_uniform: {camera_uniform:?}");

        let device = &state.device;
        // Something something... fragment shader... set colors? >_<
        let persistent = self.persistent.as_ref().unwrap();
        let shader = &persistent.shader;
        let vertex_buffer = &persistent.vertex_buffer;
        let index_buffer = &persistent.index_buffer;

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: [camera_uniform].as_bytes(),
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

        let num_indices = INDICES.len() as u32;

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
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
            render_pass.draw(0..3, 0..1);
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
