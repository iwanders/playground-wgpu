use std::io::Cursor;
use std::sync::Arc;
mod camera;

use anyhow::Context;
use glam::{Mat4, Vec3, Vec3A, Vec4, vec3, vec3a, vec4};
use log::*;
use simple_start::State;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CopyImageToBufferInfo, RenderPassBeginInfo, SubpassBeginInfo,
    SubpassContents, SubpassEndInfo,
};
use vulkano::command_buffer::{ClearColorImageInfo, CommandBufferUsage};
use vulkano::format::Format;
use vulkano::format::{ClearColorValue, ClearValue};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::sync::{self, GpuFuture};
use zerocopy::IntoBytes;
use zerocopy_derive::Immutable;

use vulkano::buffer::BufferContents;
#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable, BufferContents, Vertex)]
struct MyVertex {
    #[format(R32G32B32A32_SFLOAT)]
    #[name("input.position")]
    position: Vec3A,
    #[format(R32G32B32A32_SFLOAT)]
    #[name("input.normal")]
    normal: Vec3A,
    #[format(R32G32B32A32_SFLOAT)]
    #[name("input.color")]
    color: Vec4,
}
simple_start::static_assert_size!(MyVertex, 3 * 4 * 4);
impl MyVertex {
    pub fn pnc(position: Vec3A, normal: Vec3A, color: Vec3A) -> Self {
        Self {
            position: vec3a(position.x, position.y, position.z),
            normal: vec3a(normal.x, normal.y, normal.z),
            color: vec4(color.x, color.y, color.z, 1.0),
        }
    }
}

struct LocalState(pub State);

impl std::ops::Deref for LocalState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl LocalState {
    pub fn draw(&self) -> anyhow::Result<()> {
        let mut cam = camera::Camera::new(self.width, self.height);
        // cam.eye = (13.0, 3.7, 0.3).into();
        // cam.target = (0.0, 6.0, 1.0).into();
        cam.eye = (1.0, 0.7, 0.5).into();
        // have it look at the origin
        cam.target = (0.0, 0.0, 0.0).into();

        #[allow(dead_code)]
        let vertices: [MyVertex; 16] = [
            // The base
            MyVertex::pnc(
                vec3a(-0.5, -0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(0.0, 0.0, 1.0),
            ),
            MyVertex::pnc(
                vec3a(0.5, -0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(1.0, 0.0, 0.0),
            ),
            MyVertex::pnc(
                vec3a(0.5, 0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(0.0, 1.0, 0.0),
            ),
            MyVertex::pnc(
                vec3a(-0.5, 0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(1.0, 0.0, 1.0),
            ),
            // Face sides have their own copy of the vertices
            // because they have a different normal vector.
            MyVertex::pnc(
                vec3a(-0.5, -0.5, -0.3),
                vec3a(0.0, -0.848, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            MyVertex::pnc(
                vec3a(0.5, -0.5, -0.3),
                vec3a(0.0, -0.848, 0.53),
                vec3a(1.0, 0.0, 1.0),
            ),
            MyVertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(0.0, -0.848, 0.53),
                vec3a(0.0, 1.0, 1.0),
            ),
            //
            MyVertex::pnc(
                vec3a(0.5, -0.5, -0.3),
                vec3a(0.848, 0.0, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            MyVertex::pnc(
                vec3a(0.5, 0.5, -0.3),
                vec3a(0.848, 0.0, 0.53),
                vec3a(1.0, 0.0, 1.0),
            ),
            MyVertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(0.848, 0.0, 0.53),
                vec3a(0.0, 1.0, 1.0),
            ),
            //
            MyVertex::pnc(
                vec3a(0.5, 0.5, -0.3),
                vec3a(0.0, 0.848, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            MyVertex::pnc(
                vec3a(-0.5, 0.5, -0.3),
                vec3a(0.0, 0.848, 0.53),
                vec3a(1.0, 1.0, 1.0),
            ),
            MyVertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(0.0, 0.848, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            //
            MyVertex::pnc(
                vec3a(-0.5, 0.5, -0.3),
                vec3a(-0.848, 0.0, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            MyVertex::pnc(
                vec3a(-0.5, -0.5, -0.3),
                vec3a(-0.848, 0.0, 0.53),
                vec3a(1.0, 0.0, 1.0),
            ),
            MyVertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(-0.848, 0.0, 0.53),
                vec3a(0.0, 1.0, 1.0),
            ),
        ];
        let indices: &[u32] = &[
            // Base
            0, 1, 2, //
            0, 2, 3, //
            // side
            4, 5, 6, //
            7, 8, 9, //
            10, 11, 12, //
            13, 14, 15,
        ];

        let mut vertices = vertices.clone();
        for x in vertices.iter_mut() {
            let angle = simple_start::get_angle_f32(0.2);
            // let angle = 0.6;
            x.position = Mat4::from_rotation_z(angle)
                .transform_point3(x.position.into())
                .into();
        }

        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        use simple_start::prelude::*;
        let index_buffer = Buffer::from_slice(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices,
        )
        .unwrap();

        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            self.device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [self.width as f32, self.height as f32],
            depth_range: 0.0..=1.0,
        };

        let pipeline = {
            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
            // which one.
            // let vs = vs.entry_point("main").unwrap();
            // let fs = fs.entry_point("main").unwrap();
            //

            let data: Vec<u32> = include_bytes!("./triangle.spv")[..]
                .chunks(4)
                .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
                .collect();
            // let spirv_things = vulkano::shader::spirv::Spirv::new(&data)?;
            let shader_object = unsafe {
                vulkano::shader::ShaderModule::new(
                    self.device.clone(),
                    vulkano::shader::ShaderModuleCreateInfo::new(&data),
                )?
            };
            // let mut vertex_spv_file = Cursor::new(&vertex_spv_bytes);
            // let mut frag_spv_file = Cursor::new(&frag_spv_file);
            let vs = shader_object.entry_point("vertexMain").unwrap();
            let fs = shader_object.entry_point("fragmentMain").unwrap();

            let vertex_input_state = MyVertex::per_vertex().definition(&vs).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];

            let layout = PipelineLayout::new(
                self.device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(self.device.clone())
                    .unwrap(),
            )
            .unwrap();

            let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

            GraphicsPipeline::new(
                self.device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    // The stages of our pipeline, we have vertex and fragment stages.
                    stages: stages.into_iter().collect(),
                    // Describes the layout of the vertex input and how should it behave.
                    vertex_input_state: Some(vertex_input_state),
                    // Indicate the type of the primitives (the default is a list of triangles).
                    input_assembly_state: Some(InputAssemblyState::default()),
                    // Set the fixed viewport.
                    viewport_state: Some(ViewportState {
                        viewports: [viewport].into_iter().collect(),
                        ..Default::default()
                    }),
                    // Ignore these for now.
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState::default(),
                    )),
                    // depth_stencil_state: Some(DepthStencilState {
                    //     depth: Some(DepthState::simple()),
                    //     ..Default::default()
                    // }),
                    depth_stencil_state: Some(DepthStencilState {
                        depth: Some(DepthState {
                            write_enable: true,
                            compare_op: vulkano::pipeline::graphics::depth_stencil::CompareOp::Less,
                        }),
                        ..Default::default()
                    }),
                    // rasterization_state: Some(RasterizationState::default()),
                    rasterization_state: Some(RasterizationState {
                        cull_mode: CullMode::Back,
                        front_face: FrontFace::CounterClockwise,
                        ..Default::default()
                    }),
                    // This graphics pipeline object concerns the first pass of the render pass.
                    subpass: Some(subpass.into()),

                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )?
        };

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let render_output_image_view = ImageView::new_default(self.image.clone()).unwrap();
        let depth_image_view = ImageView::new_default(self.depth_image.clone()).unwrap();

        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                // Attach the offscreen image to the framebuffer.
                attachments: vec![render_output_image_view.clone(), depth_image_view.clone()],
                ..Default::default()
            },
        )
        .unwrap();

        #[repr(C)]
        #[derive(Copy, Clone, Debug, IntoBytes, Immutable, BufferContents)]
        struct FramePush {
            camera: Mat4,
        }

        let push_val = FramePush {
            camera: cam.to_view_projection_matrix(),
        };
        let push_constants = push_val;
        unsafe {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![
                            Some([0.0, 0.0, 1.0, 0.1].into()),
                            Some(ClearValue::Depth(1.000)),
                        ],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )?
                // new stuff
                .bind_pipeline_graphics(pipeline.clone())?
                .bind_vertex_buffers(0, vertex_buffer.clone())?
                .bind_index_buffer(index_buffer.clone())?
                .push_constants(pipeline.layout().clone(), 0, push_constants)?
                .draw_indexed(indices.len() as u32, 1, 0, 0, 0)?
                // .draw(vertices.len() as u32, 1, 0, 0)?
                .end_render_pass(SubpassEndInfo::default())?;
        }

        let command_buffer = builder.build().unwrap();
        sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .flush()
            .unwrap();

        Ok(())
    }
}

fn run_main() -> std::result::Result<(), anyhow::Error> {
    // let state = LocalState(State::new(256, 256)?);
    // state.draw()?;
    // state.save("/tmp/triangle_2d.png")?;

    let mut state = LocalState(State::new(256, 256)?);
    state.draw()?;
    state.save("/tmp/triangle_2d.png")?;
    Ok(())
}

pub fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Info)
        // .filter_level(log::LevelFilter::max())
        .try_init()?;
    run_main()?;
    println!("Hello, world! ");
    Ok(())
}
