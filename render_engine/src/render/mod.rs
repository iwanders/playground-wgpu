// A material is effectively a render pipeline.

// https://github.com/gfx-rs/wgpu/pull/754

pub struct PhongLikeMaterialConfig {
    pub rgba_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
}

struct PhongLikeGroups {}

/// A phong-shading like material. Not quite... because I made a mess.
pub struct PhongLikeMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl PhongLikeMaterial {
    pub fn new(context: &crate::Context, config: &PhongLikeMaterialConfig) -> Self {
        let render_pipeline = Self::generate_pipeline(context, config, None);
        PhongLikeMaterial { render_pipeline }
    }

    fn retrieve_embedded_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
        let config = wgpu::ShaderModuleDescriptorPassthrough {
            label: Some("shader.spv"),
            // spirv: None,
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
    }

    fn generate_pipeline(
        context: &crate::Context,
        config: &PhongLikeMaterialConfig,
        shader: Option<wgpu::ShaderModule>,
    ) -> wgpu::RenderPipeline {
        let device = &context.device;
        let shader = shader.unwrap_or_else(|| Self::retrieve_embedded_shader(device));

        let mesh_layout = device.create_bind_group_layout(&crate::mesh::GpuMesh::MESH_LAYOUT);
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let light_layout =
            device.create_bind_group_layout(&crate::lights::CpuLights::bind_group_layout());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_layout, &light_layout, &mesh_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertexMain"),
                // buffers: &[],
                buffers: &[crate::mesh::GpuMesh::get_vertex_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragmentMain"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.rgba_format,
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
                format: config.depth_format,
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
            multiview: None,
            cache: None,
        });
        render_pipeline
    }
}
