use std::io::Cursor;

use anyhow::Context;
use ash::{Entry, vk};
use log::*;
use simple_start::State;
use zerocopy::{Immutable, IntoBytes};
mod camera;

struct LocalState(pub State);

impl std::ops::Deref for LocalState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
use glam::{Mat4, Vec3, Vec3A, Vec4, vec3, vec3a, vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
struct Vertex {
    position: Vec3A,
    normal: Vec3A,
    color: Vec4,
}
simple_start::static_assert_size!(Vertex, 3 * 4 * 4);

impl Vertex {
    pub fn pnc(position: Vec3A, normal: Vec3A, color: Vec3A) -> Self {
        Self {
            position: vec3a(position.x, position.y, position.z),
            normal: vec3a(normal.x, normal.y, normal.z),
            color: vec4(color.x, color.y, color.z, 1.0),
        }
    }
}

fn make_clear_rgba(r: f32, g: f32, b: f32, a: f32) -> vk::ClearValue {
    let mut res = vk::ClearValue::default();
    unsafe {
        // res.uint32[0] = 0x3F490E7F; // 0.78 as float value, 0x7f in u8 value.
        res.color.float32[0] = r;
        res.color.float32[1] = g;
        res.color.float32[2] = b;
        res.color.float32[3] = a;
    }
    res
}

fn make_clear_depth() -> vk::ClearValue {
    let mut res = vk::ClearValue::default();
    // res.uint32[0] = 0x3F490E7F; // 0.78 as float value, 0x7f in u8 value.
    res.depth_stencil.depth = 1.0;
    res.depth_stencil.stencil = 0;
    res
}

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
struct FramePush {
    camera: Mat4,
}

impl LocalState {
    pub fn draw(&self) -> anyhow::Result<()> {
        #[allow(dead_code)]
        let vertices: [Vertex; 16] = [
            // The base
            Vertex::pnc(
                vec3a(-0.5, -0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(0.0, 0.0, 1.0),
            ),
            Vertex::pnc(
                vec3a(0.5, -0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(1.0, 0.0, 0.0),
            ),
            Vertex::pnc(
                vec3a(0.5, 0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(0.0, 1.0, 0.0),
            ),
            Vertex::pnc(
                vec3a(-0.5, 0.5, -0.3),
                vec3a(0.0, -1.0, 0.0),
                vec3a(1.0, 0.0, 1.0),
            ),
            // Face sides have their own copy of the vertices
            // because they have a different normal vector.
            Vertex::pnc(
                vec3a(-0.5, -0.5, -0.3),
                vec3a(0.0, -0.848, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            Vertex::pnc(
                vec3a(0.5, -0.5, -0.3),
                vec3a(0.0, -0.848, 0.53),
                vec3a(1.0, 0.0, 1.0),
            ),
            Vertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(0.0, -0.848, 0.53),
                vec3a(0.0, 1.0, 1.0),
            ),
            //
            Vertex::pnc(
                vec3a(0.5, -0.5, -0.3),
                vec3a(0.848, 0.0, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            Vertex::pnc(
                vec3a(0.5, 0.5, -0.3),
                vec3a(0.848, 0.0, 0.53),
                vec3a(1.0, 0.0, 1.0),
            ),
            Vertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(0.848, 0.0, 0.53),
                vec3a(0.0, 1.0, 1.0),
            ),
            //
            Vertex::pnc(
                vec3a(0.5, 0.5, -0.3),
                vec3a(0.0, 0.848, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            Vertex::pnc(
                vec3a(-0.5, 0.5, -0.3),
                vec3a(0.0, 0.848, 0.53),
                vec3a(1.0, 1.0, 1.0),
            ),
            Vertex::pnc(
                vec3a(0.0, 0.0, 0.5),
                vec3a(0.0, 0.848, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            //
            Vertex::pnc(
                vec3a(-0.5, 0.5, -0.3),
                vec3a(-0.848, 0.0, 0.53),
                vec3a(1.0, 1.0, 0.0),
            ),
            Vertex::pnc(
                vec3a(-0.5, -0.5, -0.3),
                vec3a(-0.848, 0.0, 0.53),
                vec3a(1.0, 0.0, 1.0),
            ),
            Vertex::pnc(
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

        let mut cam = camera::Camera::new(self.width, self.height);
        // cam.eye = (13.0, 3.7, 0.3).into();
        // cam.target = (0.0, 6.0, 1.0).into();
        cam.eye = (1.0, 0.7, 0.5).into();
        // have it look at the origin
        cam.target = (0.0, 0.0, 0.0).into();
        // let camera_mat = cam.to_view_projection_matrix();
        unsafe {
            // let device = self.ctx.device.lock();
            let device_memory_properties = self.ctx.get_physical_device_memory_properties();
            let device = self.ctx.device.lock();
            // Lets build a pipeline!
            // https://github.com/SaschaWillems/Vulkan/blob/b9f0ac91d2adccc3055a904d3a8f6553b10ff6cd/examples/renderheadless/renderheadless.cpp#L508
            // https://github.com/KhronosGroup/Vulkan-Samples/blob/97fcdeecf2db26a78b432b285af3869a65bb00bd/samples/extensions/dynamic_rendering/dynamic_rendering.cpp#L301
            // https://github.com/ash-rs/ash/blob/0.38.0/ash-examples/src/bin/triangle.rs#L224C1-L230
            let constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .size(std::mem::size_of::<FramePush>() as u32);
            let layout_create_info = vk::PipelineLayoutCreateInfo::default()
                .push_constant_ranges(std::slice::from_ref(&constant_range));
            let layout = device.create_pipeline_layout(&layout_create_info, None)?;

            let cache_info = vk::PipelineCacheCreateInfo::default();
            let pipeline_cache = device.create_pipeline_cache(&cache_info, None)?;

            let index_buffer_data = indices;
            let index_buffer_info = vk::BufferCreateInfo::default()
                .size((index_buffer_data.len() * std::mem::size_of::<u32>()) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let index_buffer = device.create_buffer(&index_buffer_info, None).unwrap();
            let index_buffer_memory_req = device.get_buffer_memory_requirements(index_buffer);
            let index_buffer_memory_index = simple_start::find_memorytype_index(
                &index_buffer_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("Unable to find suitable memorytype for the index buffer.");

            let index_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: index_buffer_memory_req.size,
                memory_type_index: index_buffer_memory_index,
                ..Default::default()
            };
            let index_buffer_memory = device.allocate_memory(&index_allocate_info, None).unwrap();
            let index_ptr = device
                .map_memory(
                    index_buffer_memory,
                    0,
                    index_buffer_memory_req.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            let mut index_slice = ash::util::Align::new(
                index_ptr,
                std::mem::align_of::<u32>() as u64,
                index_buffer_memory_req.size,
            );
            index_slice.copy_from_slice(&index_buffer_data);
            device.unmap_memory(index_buffer_memory);
            device
                .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
                .unwrap();

            let vertex_input_buffer_info = vk::BufferCreateInfo {
                size: vertices.len() as u64 * std::mem::size_of::<Vertex>() as u64,
                usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };

            let vertex_input_buffer = device
                .create_buffer(&vertex_input_buffer_info, None)
                .unwrap();

            let vertex_input_buffer_memory_req =
                device.get_buffer_memory_requirements(vertex_input_buffer);

            let vertex_input_buffer_memory_index = simple_start::find_memorytype_index(
                &vertex_input_buffer_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("Unable to find suitable memorytype for the vertex buffer.");

            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo {
                allocation_size: vertex_input_buffer_memory_req.size,
                memory_type_index: vertex_input_buffer_memory_index,
                ..Default::default()
            };

            let vertex_input_buffer_memory = device
                .allocate_memory(&vertex_buffer_allocate_info, None)
                .unwrap();

            let vert_ptr = device
                .map_memory(
                    vertex_input_buffer_memory,
                    0,
                    vertex_input_buffer_memory_req.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();

            let mut vert_align = ash::util::Align::new(
                vert_ptr,
                std::mem::align_of::<Vertex>() as u64,
                vertex_input_buffer_memory_req.size,
            );
            vert_align.copy_from_slice(&vertices);
            device.unmap_memory(vertex_input_buffer_memory);
            device
                .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
                .unwrap();

            const COMPILE_SHADERS: bool = true;
            const USE_SLANG_SHADER: bool = true;

            let (vertex_spv_bytes, frag_spv_file): (Vec<u8>, Vec<u8>) = if COMPILE_SHADERS {
                if USE_SLANG_SHADER {
                    let data = include_bytes!("./triangle.spv")[..].to_vec();
                    (data.clone(), data)
                } else {
                    // let vert_bytes = simple_start::shader_util::compile_shader(
                    //     include_str!("./triangle.vert"),
                    //     naga::ShaderStage::Vertex,
                    // )?;
                    // let frag_bytes = simple_start::shader_util::compile_shader(
                    //     include_str!("./triangle.frag"),
                    //     naga::ShaderStage::Fragment,
                    // )?;
                    // use zerocopy::IntoBytes;
                    // std::fs::write("/tmp/foo.bin", &z.as_bytes()).expect("Unable to write file");
                    (todo!("missing shader file"), todo!("missing shader file"))
                }
            } else {
                // spv files from https://github.com/ash-rs/ash/tree/0.38.0/ash-examples/shader/triangle
                (
                    todo!("missing shader file"),
                    todo!("missing shader file"),
                    // include_bytes!("./vert.spv")[..].to_vec(),
                    // include_bytes!("./frag.spv")[..].to_vec(),
                )
            };
            let mut vertex_spv_file = Cursor::new(&vertex_spv_bytes);
            let mut frag_spv_file = Cursor::new(&frag_spv_file);

            let vertex_code = ash::util::read_spv(&mut vertex_spv_file)
                .expect("Failed to read vertex shader spv file");
            let vertex_shader_info = vk::ShaderModuleCreateInfo::default().code(&vertex_code);

            let frag_code = ash::util::read_spv(&mut frag_spv_file)
                .expect("Failed to read fragment shader spv file");
            let frag_shader_info = vk::ShaderModuleCreateInfo::default().code(&frag_code);

            let vertex_shader_module = device
                .create_shader_module(&vertex_shader_info, None)
                .expect("Vertex shader module error");

            let fragment_shader_module = device
                .create_shader_module(&frag_shader_info, None)
                .expect("Fragment shader module error");

            let (vert_shader_name, frag_shader_name) = if USE_SLANG_SHADER {
                let vertex_name = std::ffi::CStr::from_bytes_with_nul_unchecked(b"vertexMain\0");
                let fragment_main =
                    std::ffi::CStr::from_bytes_with_nul_unchecked(b"fragmentMain\0");
                (vertex_name, fragment_main)
            } else {
                let main = std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0");
                (main, main)
            };
            let shader_stage_create_infos = [
                vk::PipelineShaderStageCreateInfo {
                    module: vertex_shader_module,
                    p_name: vert_shader_name.as_ptr(),
                    stage: vk::ShaderStageFlags::VERTEX,
                    ..Default::default()
                },
                vk::PipelineShaderStageCreateInfo {
                    module: fragment_shader_module,
                    p_name: frag_shader_name.as_ptr(),
                    stage: vk::ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ];
            let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            }];
            let vertex_input_attribute_descriptions = [
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: std::mem::offset_of!(Vertex, position) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: std::mem::offset_of!(Vertex, normal) as u32,
                },
                vk::VertexInputAttributeDescription {
                    location: 2,
                    binding: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT,
                    offset: std::mem::offset_of!(Vertex, color) as u32,
                },
            ];

            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
                .vertex_binding_descriptions(&vertex_input_binding_descriptions);
            let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,

                ..Default::default()
            };
            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.width as f32,
                height: self.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];
            let scissors = [vk::Rect2D::default().extent(
                vk::Extent2D::default()
                    .height(self.height)
                    .width(self.width),
            )];
            let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
                .scissors(&scissors)
                .viewports(&viewports);

            let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
                front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                line_width: 1.0,
                polygon_mode: vk::PolygonMode::FILL,
                cull_mode: vk::CullModeFlags::NONE,

                ..Default::default()
            };
            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
                rasterization_samples: vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            };
            let noop_stencil_state = vk::StencilOpState {
                fail_op: vk::StencilOp::KEEP,
                pass_op: vk::StencilOp::KEEP,
                depth_fail_op: vk::StencilOp::KEEP,
                compare_op: vk::CompareOp::ALWAYS,
                ..Default::default()
            };
            let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
                depth_test_enable: 1,
                depth_write_enable: 1,
                depth_compare_op: vk::CompareOp::LESS,
                front: noop_stencil_state,
                back: noop_stencil_state,
                // min_depth_bounds: 0.0,
                // max_depth_bounds: 1.0,
                ..Default::default()
            };
            let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::RGBA,
            }];
            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);

            let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_state);

            let mut rendering_create = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(std::slice::from_ref(&vk::Format::R8G8B8A8_UNORM))
                .depth_attachment_format(vk::Format::D32_SFLOAT);
            // .stencil_attachment_format(vk::Format::D32_SFLOAT);

            let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .push_next(&mut rendering_create)
                .stages(&shader_stage_create_infos)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .depth_stencil_state(&depth_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(layout);

            let pipelines = device
                .create_graphics_pipelines(
                    pipeline_cache,
                    std::slice::from_ref(&graphic_pipeline_info),
                    None,
                )
                .map_err(|(_, e)| e)
                .with_context(|| "failed to create pipeline")?;
            let pipeline = pipelines[0];
            //
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            // Shoot I need a view now.
            let create_view_info = vk::ImageViewCreateInfo::default()
                .image(self.image.image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let image_view = device.create_image_view(&create_view_info, None)?;

            let create_depth_view = vk::ImageViewCreateInfo::default()
                .image(self.depth_image.image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::D32_SFLOAT)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let depth_view = device.create_image_view(&create_depth_view, None)?;
            drop(device);
            let writer = self
                .ctx
                .record_command_buffer(&self.draw_command_buffer, &command_buffer_begin_info)?;
            // self.device
            //     .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)?;

            // https://github.com/KhronosGroup/Vulkan-Samples/blob/97fcdeecf2db26a78b432b285af3869a65bb00bd/samples/extensions/dynamic_rendering_local_read/dynamic_rendering_local_read.cpp#L878C39-L878C60

            let clear_value = make_clear_rgba(1.0, 0.0, 0.0, 0.2);

            let color_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image_view(image_view)
                .resolve_mode(vk::ResolveModeFlags::NONE)
                .load_op(vk::AttachmentLoadOp::CLEAR) // This should be clear to actually clear it.
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear_value);
            // let mut color_attachments = [color_attachment_info; 4];
            let depth_attachment_info = vk::RenderingAttachmentInfo::default()
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .image_view(depth_view)
                .resolve_mode(vk::ResolveModeFlags::NONE)
                .load_op(vk::AttachmentLoadOp::CLEAR) // This should be clear to actually clear it.
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(make_clear_depth());

            let rendering_info = vk::RenderingInfo::default()
                .layer_count(1)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D {
                        width: self.width,
                        height: self.height,
                    },
                })
                .color_attachments(std::slice::from_ref(&color_attachment_info))
                .depth_attachment(&depth_attachment_info);

            // do things.
            writer.imagebuf_layout_barrier(
                &self.draw_command_buffer,
                &self.image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );

            writer.imagebuf_layout_barrier(
                &self.draw_command_buffer,
                &self.depth_image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            );

            let viewport = vk::Viewport::default()
                .width(self.width as f32)
                .height(self.height as f32)
                .min_depth(0.0)
                .max_depth(1.0);

            writer.cmd_set_viewport(self.draw_command_buffer, 0, std::slice::from_ref(&viewport));

            let scissors = vk::Rect2D::default().extent(
                vk::Extent2D::default()
                    .width(self.width)
                    .height(self.height),
            );
            writer.cmd_set_scissor(self.draw_command_buffer, 0, std::slice::from_ref(&scissors));
            writer.cmd_bind_vertex_buffers(
                self.draw_command_buffer,
                0,
                &[vertex_input_buffer],
                &[0],
            );
            writer.cmd_bind_index_buffer(
                self.draw_command_buffer,
                index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            writer.cmd_bind_pipeline(
                self.draw_command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
            writer.cmd_begin_rendering(self.draw_command_buffer, &rendering_info);

            // send the push constants.
            //
            let pushdata = FramePush {
                camera: cam.to_view_projection_matrix(),
            };

            writer.cmd_push_constants(
                self.draw_command_buffer,
                layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                pushdata.as_bytes(),
            );

            // Oh, we still do need a pipeline here... just no render passess.
            // https://github.com/KhronosGroup/Vulkan-Samples/blob/97fcdeecf2db26a78b432b285af3869a65bb00bd/samples/extensions/dynamic_rendering/dynamic_rendering.cpp
            const DRAW_INDICED: bool = true;
            if DRAW_INDICED {
                writer.cmd_draw_indexed(self.draw_command_buffer, indices.len() as _, 1, 0, 0, 0);
            } else {
                writer.cmd_draw(self.draw_command_buffer, vertices.len() as _, 1, 0, 0);
            }
            writer.cmd_end_rendering(self.draw_command_buffer);
            // self.device.end_command_buffer(self.draw_command_buffer)?;
            writer.finish(&self.draw_command_buffer)?;

            let command_buffers = vec![self.draw_command_buffer];
            let device = self.ctx.device.lock();
            device.reset_fences(&[self.draw_commands_reuse_fence])?;

            let sema = [self.rendering_complete_semaphore];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&sema)
                .wait_dst_stage_mask(&[])
                .command_buffers(&command_buffers)
                .signal_semaphores(&[]);
            device.queue_submit(self.queue, &[submit_info], self.draw_commands_reuse_fence)?;
            let timeout = 1_000_000; // in nanoseconds.
            device.wait_for_fences(&[self.draw_commands_reuse_fence], true, timeout)?;
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Ok(())
    }
}

fn run_main() -> std::result::Result<(), anyhow::Error> {
    let state = LocalState(State::new(512, 512)?);
    state.draw()?;
    state.save("/tmp/first_3d.png")?;

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
