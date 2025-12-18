use glam::{Mat4, Vec3, vec3};
use log::*;
use simple_start::State;
use wgpu::util::DeviceExt;

// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/
// Skipping over textures and bindgroups
// https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/

// https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/3d-meshes/a-simple-example.html

struct LocalState {
    width: u32,
    height: u32,
}
impl LocalState {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
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

struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}
impl Camera {
    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_lh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        // 3. I dropped the opengl matrix that precded this multiplication.
        return proj * view;
    }
    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.build_view_projection_matrix(),
        }
    }
}
// That means that in normalized device coordinates (opens new window), the x-axis and y-axis are in the range of -1.0 to +1.0, and the z-axis is 0.0 to +1.0.

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
struct CameraUniform {
    // ~We can't use cgmath with bytemuck directly, so we'll have~
    // we use glam so we can.
    view_proj: Mat4,
}

impl simple_start::Drawable for LocalState {
    fn render(&mut self, state: &mut State) -> Result<(), wgpu::SurfaceError> {
        state.window.as_ref().map(|k| k.request_redraw());

        // We can't render unless the surface is configured
        if !state.is_surface_configured {
            return Err(wgpu::SurfaceError::Lost);
        }
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.7, 0.0).into(),
            // have it look at the origin
            target: (0.0, 6.0, -1.0).into(),
            // which way is "up"
            up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            aspect: self.width as f32 / self.height as f32,
            fovy: 90.0,
            znear: 0.001,
            zfar: 1000.0,
        };
        let camera_uniform = camera.to_uniform();
        warn!("camera_uniform: {camera_uniform:?}");

        // Something something... fragment shader... set colors? >_<
        let device = &state.device;
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: [camera_uniform].as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut vs = VERTICES;
        for x in vs.iter_mut() {
            // let angle = simple_start::get_angle_f32(0.2);
            let angle = 0.6;
            x.position = Mat4::from_rotation_z(angle).transform_point3(x.position);
        }

        pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

        let depth_size = wgpu::Extent3d {
            // 2.
            width: self.width.max(1),
            height: self.height.max(1),
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
        /*
        // We only need the depth sampler for sampling textures.
        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        */

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
        let num_indices = INDICES.len() as u32;

        let texture_format = state.texture_view.texture().format();
        let extent = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

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

        if state.surface.is_none() {
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &state.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &state.buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(self.width * std::mem::size_of::<u32>() as u32),
                        rows_per_image: Some(self.width),
                    },
                },
                extent,
            );
        }
        info!("running queue");

        state.queue.submit(Some(encoder.finish()));

        // And copy from the surface to the window canvas.
        /**/
        if let Some(output) = surface_texture {
            /*
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &state.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &output.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::Extent3d {
                    width: output.texture.width(),
                    height: output.texture.height(),
                    depth_or_array_layers: output.texture.depth_or_array_layers(),
                },
            );
            state.queue.submit(Some(encoder.finish()));*/
            output.present();
        }
        Ok(())
    }
}
async fn async_main() -> std::result::Result<(), anyhow::Error> {
    if true {
        let drawable = LocalState::new(1024, 768);
        simple_start::async_render(drawable, 1024, 768, "/tmp/first_3d.png").await?;
    }
    let drawable = LocalState::new(800, 600);
    simple_start::async_main(drawable).await;

    Ok(())
}

pub fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::builder()
        .is_test(false)
        // .filter_level(log::LevelFilter::Warn)
        .filter_level(log::LevelFilter::max())
        .try_init()?;
    pollster::block_on(async_main())?;
    println!("Hello, world! ");
    Ok(())
}
