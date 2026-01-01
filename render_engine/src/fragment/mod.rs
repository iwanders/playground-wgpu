// ~A material is effectively a render pipeline.~
//
// A material may be made up of a bunch of pipelines
//  All draws associated to the same pipeline can be done in one pass?
// Pipelines are ran in order, with objects that are part of the pipeline being drawn?
// One pass can use multiple pipelines.
//
// How do we use this multi draw?
// https://github.com/gfx-rs/wgpu/pull/754
// On render bundles; https://toji.dev/webgpu-best-practices/render-bundles.html#resource-updates
//   Commands are static, data in them is not!
//   More on indirect draws
//   https://toji.dev/webgpu-best-practices/indirect-draws
//     -> compute culling; https://vkguide.dev/docs/gpudriven/compute_culling/
//
//   Oh, hmm, we can bind buffer array as well; https://docs.rs/wgpu/latest/wgpu/enum.BindingResource.html#variant.BufferArray hmm
//
// helpful:
//   https://github.com/gfx-rs/wgpu-rs/issues/18#issuecomment-499362497
//   https://github.com/gfx-rs/wgpu-rs/issues/18#issuecomment-499550688
//
// Most objects can share the vertex stage, but will have a different fragment stage?
// Mesh shaders output vertices, but could still share the fragment shaders.
// Should split out the stages.
//
//
// Okay, so a single render pass can use multiple pipelines, but pipelines are expensive so we want swap them as little
// as possible.
//
// We want to structure the overarching system by:
//    - Render pass.
//      - n pipelines
//        - geometries per pipeline.
//
// Render pass need not be sequential, they can have arbitrary parents and attachnment inputs.
//
// This also allows us to have a 'postprocess' render pass that works in screen space over a viewport wide rectangle?
// For a set of geometries to be drawn, they thus need:
//  A list of (render_pass, pipeline) combinations.
//
// The actual renderable is the output of a vertex buffer;
//  - Mesh shaders output vertex data based on... nothing or arbitrary data input.
//  - Vertex shader + polygons output vertex data.
//
//  - The pipeline specifies how the vertex data is produced AND which fragment shader is used.
//  - A compute pipeline & buffer would be nice to fit in as arbitrary passess as well... hmm.
//
//  - Lights is technically a property of the fragment shader?
//
// Pass always uses the same color & depth attachments.
//
// For the render:
//  We record the command buffers with the current data.
//  We submit the command buffers.

// Not sold on this.

/*
pub struct RenderPassId(usize);
pub struct RenderPipelineId(usize);

pub struct SimpleRenderableMesh {
    mesh: super::mesh::GpuMesh,
    part_of: (RenderPassId, RenderPipelineId),
}

pub trait Renderable {
    fn is_part_of(&self, pass_id: RenderPassId, pipeline_id: RenderPipelineId) -> bool;
    fn add_commands(&self, our_pass: &mut wgpu::RenderPass);
}

pub struct RenderPipeline {
    id: RenderPipelineId,
    pipeline: wgpu::RenderPipeline,
}

// Practically an owned RenderPassDescriptor, with some extras
pub struct RenderPass {
    id: RenderPassId,
    pub label: Option<String>,
    pipelines: Vec<RenderPipeline>,
    color_attachments: Vec<()>,
    depth_attachment: Option<()>,
}

pub struct Renderer {
    passess: Vec<RenderPass>,
}

impl Renderer {
    pub fn add_to_encoder(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        renderables: &[&dyn Renderable],
    ) {
        for this_pass in self.passess.iter() {
            let encode_pass = encoder.begin_render_pass(todo!());
            for pipeline in this_pass.pipelines.iter() {
                encode_pass.set_pipeline(&pipeline.pipeline);

                // Set camera & lights bind.

                // Then do the renderables.
                for r in renderables.iter() {
                    if r.is_part_of(this_pass.id, pipeline.id) {
                        r.add_commands(&mut encode_pass);
                    }
                }
            }
        }
    }
}*/

use crate::wgpu_util::StaticWgslStack;

pub const MESH_OBJECT_WGSL: StaticWgslStack = StaticWgslStack {
    name: "phong_thing",
    entry: "main",
    sources: &[
        include_str!("../shader_common.wgsl"),
        include_str!("shader.wgsl"),
    ],
};

pub mod mesh_object_textured;

pub struct PBRMaterialConfig {
    pub rgba_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
}

/// A phong-shading like material. Not quite... because I made a mess.
pub struct PBRMaterial {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl PBRMaterial {
    pub fn new(
        context: &crate::Context,
        config: &PBRMaterialConfig,
        vertex_source: crate::vertex::VertexCreaterShader,
    ) -> Self {
        let render_pipeline = Self::generate_pipeline(context, config, vertex_source);
        PBRMaterial { render_pipeline }
    }

    fn retrieve_embedded_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
        MESH_OBJECT_WGSL.create(device)
    }

    fn generate_pipeline(
        context: &crate::Context,
        config: &PBRMaterialConfig,
        vertex_source: crate::vertex::VertexCreaterShader,
    ) -> wgpu::RenderPipeline {
        let device = &context.device;
        let fragment_shader = Self::retrieve_embedded_shader(device);

        let mesh_layout =
            device.create_bind_group_layout(&crate::vertex::mesh_object::MeshObject::MESH_LAYOUT);
        let camera_layout =
            device.create_bind_group_layout(&crate::view::ViewUniform::bind_group_layout());
        let light_layout =
            device.create_bind_group_layout(&crate::lights::CpuLights::bind_group_layout());

        let texture_layout =
            device.create_bind_group_layout(&crate::texture::GpuTextureInfo::bind_group_layout());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_layout, &light_layout, &mesh_layout, &texture_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_source.shader_module,
                entry_point: Some(&vertex_source.entry),
                // buffers: &[],
                buffers: &[crate::vertex::mesh::GpuMesh::get_vertex_layout()],
                // compilation_options: Default::default(),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: Some("main"),
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
