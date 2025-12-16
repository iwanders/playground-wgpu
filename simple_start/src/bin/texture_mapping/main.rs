use glam::{Mat4, Vec3, vec3};
use log::*;
use simple_start::State;
use wgpu::util::DeviceExt;

// https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/texturing/texture-mapping.html

struct LocalState(pub State);

impl std::ops::Deref for LocalState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
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

const VERTICES: [Vertex; 4] = [
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
];
const INDICES: &[u16] = &[
    // Base
    0, 1, 2, //
    0, 2, 3, //
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

impl LocalState {
    // https://github.com/gfx-rs/gfx/tree/master/src/backend/dx12#normalized-coordinates
    pub async fn draw(&self) -> anyhow::Result<()> {
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (-0.1, 0.4, 0.6).into(),
            // have it look at the origin
            target: (-0.6, 1.5, 3.0).into(),
            // which way is "up"
            up: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            aspect: self.width as f32 / self.height as f32,
            fovy: 90.0,
            znear: 0.001,
            zfar: 1000.0,
        };
        let camera_uniform = camera.to_uniform();
        warn!("camera_uniform: {camera_uniform:?}");

        let device = &self.device;
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

        // Make the texture
        let texture_size = wgpu::Extent3d {
            width: 256,
            height: 256,
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });
        let mut diffuse_rgba = vec![0u8; (texture_size.width * texture_size.height * 4) as usize];
        for i in 0..texture_size.width {
            for j in 0..texture_size.height {
                let offset = 4 * (j as usize * texture_size.width as usize + i as usize);
                if false {
                    diffuse_rgba[offset] = i as u8; // r
                    diffuse_rgba[offset + 1] = j as u8; // g
                    diffuse_rgba[offset + 2] = 128; // b
                } else {
                    diffuse_rgba[offset + 0] = if (i / 16) % 2 == (j / 16) % 2 { 255 } else { 0 }; // r
                    diffuse_rgba[offset + 1] = if ((i.wrapping_sub(j)) / 16) % 2 == 0 {
                        255
                    } else {
                        0
                    }; // g
                    diffuse_rgba[offset + 2] = if ((i + j) / 16) % 2 == 0 { 255 } else { 0 }; // b
                }
                diffuse_rgba[offset + 3] = 255; // a
            }
        }
        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
            texture_size.width,
            texture_size.height,
            diffuse_rgba.clone(),
        )
        .unwrap();
        buffer.save("/tmp/texture.png").unwrap();

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );
        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // // /Make the texture

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

        let texture_format = self.texture_view.texture().format();
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            });
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
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

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
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
            render_pass.set_bind_group(1, &texture_bind_group, &[]); // NEW!
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
            render_pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width * std::mem::size_of::<u32>() as u32),
                    rows_per_image: Some(self.width),
                },
            },
            extent,
        );
        info!("running queue");
        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

async fn async_main() -> std::result::Result<(), anyhow::Error> {
    let state = LocalState(State::new(512, 512).await?);
    state.draw().await?;
    state.save("/tmp/texture_mapping.png").await?;

    Ok(())
}

pub fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Warn)
        // .filter_level(log::LevelFilter::max())
        .try_init()?;
    pollster::block_on(async_main())?;
    println!("Hello, world! ");
    Ok(())
}
