// I started with [this](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/), but that started setting up
// application state while we should be able to just render to an image??
//
// Oh, that is in https://sotrh.github.io/learn-wgpu/showcase/windowless/ so yeah I was on the right track
// with just the adapter, device and queue.
//
// Lets just start with tutorial 2, and pick from tutorial 1 as we see fit.

// https://github.com/ash-rs/ash/blob/b724b78dac8d83879ed7a1aad2b91bb9f2beb5cf/ash-examples/src/lib.rs
// https://vulkan-tutorial.com/Overview
//
// hmm
// https://github.com/Traverse-Research/gpu-allocator
//
//
// Vulkanised 2024: Common Mistakes When Learning Vulkan - Charles Giesse
//  https://youtu.be/0OqJtPnkfC8
// Abstract over what you need, don't lose it.
// Read up on what dynamic rendering in the vk api is, makes things consistent.
//  https://youtu.be/0OqJtPnkfC8?t=1038
//  https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/03_Drawing/00_Framebuffers.html
// Don't support swapchain resizing.
// Understand vkPresentMode
// do not rely on vkQueueWaitIdle, it does not eliminate pipelining.
//
// Vulkanised 2025: So You Want to Write a Vulkan Renderer in 2025 - Charles Giessen
//      https://www.youtube.com/watch?v=7CtjMfDdTdg
// buffer device address; use pointers without descriptors.
// use scalar block layout, mirrors the C layout.
// timeline semaphores, monotonically increasing u64, incremented when work is done, can wait on it to be value.
//  https://docs.vulkan.org/refpages/latest/refpages/source/vkWaitSemaphores.html
//
use anyhow::Context;
use ash::ext::debug_utils;
use ash::{Entry, vk};
use log::*;
use std::ffi;
use std::path::Path;
use zerocopy::IntoBytes;
pub struct State {
    instance: ash::Instance,
    // surface: wgpu::Surface<'static>,
    pub device: ash::Device,
    pub pdevice: ash::vk::PhysicalDevice,
    pub queue: vk::Queue,
    pub pool: vk::CommandPool,
    pub setup_command_buffer: vk::CommandBuffer,
    pub draw_command_buffer: vk::CommandBuffer,
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub draw_commands_reuse_fence: vk::Fence,
    pub setup_commands_reuse_fence: vk::Fence,
    pub rendering_complete_semaphore: vk::Semaphore,
    // pub queue: wgpu::Queue,
    // pub buffer: wgpu::Buffer,
    // pub texture: wgpu::Texture,
    // pub texture_view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    unsafe {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            std::borrow::Cow::from("")
        } else {
            ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            std::borrow::Cow::from("")
        } else {
            ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };
        if message_severity == ash::vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE {
            trace!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}");
        } else if message_severity == ash::vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
            info!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}");
        } else if message_severity == ash::vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
            warn!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}");
        } else if message_severity == ash::vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
            error!("{message_type:?} [{message_id_name} ({message_id_number})] : {message}");
        } else {
            error!(
                "UNHANDLED LEVEL: {message_type:?} [{message_id_name} ({message_id_number})] : {message}"
            );
        }
    }
    vk::FALSE
}

// Helper copied verbatim from https://github.com/ash-rs/ash/blob/0.38.0/ash-examples/src/lib.rs#L122
// This video: https://youtu.be/nD83r06b5NE?t=1588
// explains why we need this... there's multipule memory heaps on the GPU O_o
pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

impl State {
    // ...
    pub async fn new(width: u32, height: u32) -> anyhow::Result<State> {
        let entry = Entry::linked();

        let app_name = unsafe { ffi::CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0") };

        // apt install vulkan-validationlayers just works.
        let layer_names: [&ffi::CStr; 1] =
            [
                unsafe {
                    ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")
                },
            ];
        let layers_names_raw: Vec<*const std::ffi::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let mut extension_names: Vec<*const i8> = vec![];

        unsafe {
            extension_names
                .push(ffi::CStr::from_bytes_with_nul_unchecked(b"VK_KHR_surface\0").as_ptr());
            // extension_names.push(
            //     ffi::CStr::from_bytes_with_nul_unchecked(b"VK_EXT_headless_surface\0").as_ptr(),
            // );
            extension_names
                .push(ffi::CStr::from_bytes_with_nul_unchecked(b"VK_EXT_debug_utils\0").as_ptr());
        }

        let appinfo = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 3, 0));

        // let mut extension_names =
        //     ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())
        //         .unwrap()
        //         .to_vec();
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(vk::InstanceCreateFlags::default());
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        dbg!();

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
        let debug_call_back =
            unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None) }.unwrap();
        // vkEnumerateInstanceExtensionProperties
        // instance.enumerate_device_extension_properties(device)
        let pdevices =
            unsafe { instance.enumerate_physical_devices() }.expect("Physical device error");

        let mut use_pdevice = None;
        let mut use_index = None;

        for pdevice in pdevices {
            if use_index.is_some() {
                break;
            }
            let props = unsafe { instance.get_physical_device_properties(pdevice) };
            info!(
                "{:?}, {:?} -> v {}.{}.{}.{}",
                std::ffi::CStr::from_bytes_until_nul(props.device_name.as_bytes()),
                props.device_type,
                ash::vk::api_version_variant(props.api_version),
                ash::vk::api_version_major(props.api_version),
                ash::vk::api_version_minor(props.api_version),
                ash::vk::api_version_patch(props.api_version),
            );
            for (index, info) in
                unsafe { instance.get_physical_device_queue_family_properties(pdevice) }
                    .iter()
                    .enumerate()
            {
                dbg!(info.queue_flags);
                let supports_graphic_and_surface =
                    info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                if supports_graphic_and_surface {
                    use_index = Some(index);
                    use_pdevice = Some(pdevice);
                    // break;
                }
            }
        }

        info!("Using device {use_pdevice:?} and device queue {use_index:?}");
        let queue_family_index = use_index.with_context(|| "could not find device")? as u32;
        let pdevice = use_pdevice.with_context(|| "could not find device")?;

        let device: ash::Device = unsafe {
            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [
                ash::khr::swapchain::NAME.as_ptr(),
                ash::khr::dynamic_rendering::NAME.as_ptr(),
                ash::khr::dynamic_rendering_local_read::NAME.as_ptr(),
            ];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [0.0];

            let mut physical_device_features =
                vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);

            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);
            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .push_next(&mut physical_device_features)
                // .enabled_features(&features)
            ;
            instance
                .create_device(pdevice, &device_create_info, None)
                .unwrap()
        };
        info!("device created");
        // let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        // let headless_instance = ash::ext::headless_surface::Instance::new(&entry, &instance);
        dbg!();

        // let surface = unsafe {
        //     let create_info = ash::vk::HeadlessSurfaceCreateInfoEXT {
        //         flags: ash::vk::HeadlessSurfaceCreateFlagsEXT::empty(),
        //         ..Default::default()
        //     };
        //     headless_instance.create_headless_surface(&create_info, None)?
        // };
        // info!("surface: {surface:?}");
        dbg!();

        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        info!("queue: {queue:?}");

        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe { device.create_command_pool(&pool_create_info, None)? };
        // Okay, now we have a queue...

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(2)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        let setup_command_buffer = command_buffers[0];
        let draw_command_buffer = command_buffers[1];

        // Next, we create the output image?

        let extent = vk::Extent3D {
            width,
            height,
            depth: 1,
        };
        let img_info = vk::ImageCreateInfo::default()
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(extent)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
            )
            .mip_levels(1)
            .array_layers(1)
            .image_type(vk::ImageType::TYPE_2D);
        let image = unsafe { device.create_image(&img_info, None)? };
        info!("image: {image:?}");
        let memory_req = unsafe { device.get_image_memory_requirements(image) };
        info!("memory_req: {memory_req:#?}");

        // Okay... we have an image now... but it doesn't have any memory allocated to it?]
        // https://youtu.be/nD83r06b5NE?t=1637
        // so yea that's... complex.
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(pdevice) };
        let image_memory_req = unsafe { device.get_image_memory_requirements(image) };
        let image_memory_index = find_memorytype_index(
            &image_memory_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .with_context(|| "Unable to find suitable memory index for image.")?;
        info!("image_memory_index: {image_memory_index:#?}");

        let image_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let image_memory = unsafe { device.allocate_memory(&image_allocate_info, None)? };
        unsafe { device.bind_image_memory(image, image_memory, 0)? };

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let draw_commands_reuse_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let setup_commands_reuse_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let rendering_complete_semaphore =
            unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        Ok(State {
            instance,
            device,
            pdevice,
            width,
            height,
            queue,
            pool,
            setup_command_buffer,
            draw_command_buffer,
            image,
            image_memory,
            draw_commands_reuse_fence,
            setup_commands_reuse_fence,
            rendering_complete_semaphore,
        })
    }

    pub async fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let p: &Path = path.as_ref();

        // extract data from the memory behind the image? >_<

        // https://github.com/SaschaWillems/Vulkan/blob/b9f0ac91d2adccc3055a904d3a8f6553b10ff6cd/examples/renderheadless/renderheadless.cpp#L691
        // oof... that's a lot of work, but it's a lot of duplication...

        let extent = vk::Extent3D {
            width: self.width,
            height: self.height,
            depth: 1,
        };
        let img_info = vk::ImageCreateInfo::default()
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(extent)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(vk::ImageUsageFlags::TRANSFER_DST) // Flag as destination now
            .mip_levels(1)
            .array_layers(1)
            // .initial_layout(vk::ImageLayout::UNDEFINED)
            .tiling(vk::ImageTiling::LINEAR)
            .image_type(vk::ImageType::TYPE_2D);
        let image = unsafe { self.device.create_image(&img_info, None)? };
        info!("image: {image:?}");
        let memory_req = unsafe { self.device.get_image_memory_requirements(image) };
        info!("memory_req: {memory_req:#?}");

        let device_memory_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.pdevice)
        };
        let image_memory_req = unsafe { self.device.get_image_memory_requirements(image) };
        let image_memory_index = find_memorytype_index(
            &image_memory_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, // !!! Allocate it as host visible, and coherent.
        )
        .with_context(|| "Unable to find suitable memory index for image.")?;
        info!("image_memory_index: {image_memory_index:#?}");

        let image_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let image_memory = unsafe { self.device.allocate_memory(&image_allocate_info, None)? };
        unsafe { self.device.bind_image_memory(image, image_memory, 0)? };

        // Something something command to transfer now...

        // Execute commands.
        unsafe {
            //
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device
                .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)?;

            {
                // Source image layout.
                let image_barrier = vk::ImageMemoryBarrier::default()
                    .image(self.image)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                self.device.cmd_pipeline_barrier(
                    self.draw_command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
                );
            }

            {
                // Destination image layout.
                let image_barrier = vk::ImageMemoryBarrier::default()
                    .image(image)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                self.device.cmd_pipeline_barrier(
                    self.draw_command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
                );
            }

            // Do some commands here...
            let region = vk::ImageCopy::default()
                .extent(extent)
                .dst_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1),
                )
                .src_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1),
                );
            self.device.cmd_copy_image(
                self.draw_command_buffer,
                self.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );

            self.device.end_command_buffer(self.draw_command_buffer)?;

            let command_buffers = vec![self.draw_command_buffer];
            self.device
                .reset_fences(&[self.draw_commands_reuse_fence])?;

            let sema = [self.rendering_complete_semaphore];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&sema)
                .wait_dst_stage_mask(&[])
                .command_buffers(&command_buffers)
                .signal_semaphores(&[]);

            self.device
                .queue_submit(self.queue, &[submit_info], self.draw_commands_reuse_fence)?;
            let timeout = 1_000_000; // in nanoseconds.
            self.device
                .wait_for_fences(&[self.draw_commands_reuse_fence], true, timeout)?;
        };

        // Then, we map the memory, and copy it...
        //
        let data = {
            let mut res = vec![];

            let subres = vk::ImageSubresource::default().aspect_mask(vk::ImageAspectFlags::COLOR);
            let layout = unsafe { self.device.get_image_subresource_layout(image, subres) };
            info!("layout: {layout:#?}");
            unsafe {
                let mut raw_location = self.device.map_memory(
                    image_memory,
                    0,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )?;
                raw_location = raw_location.offset(layout.offset as isize);
                let mut raw_data =
                    std::slice::from_raw_parts(raw_location.cast::<u8>(), layout.size as usize);
                info!("raw len: {:?}", raw_data.len());
                // https://github.com/SaschaWillems/Vulkan/blob/b9f0ac91d2adccc3055a904d3a8f6553b10ff6cd/examples/renderheadless/renderheadless.cpp#L801-L816
                for y in 0..self.height {
                    for x in 0..self.width as usize {
                        let p = raw_data[x * 4..(x + 1) * 4].as_bytes();
                        res.push(p[0]);
                        res.push(p[1]);
                        res.push(p[2]);
                        res.push(p[3]);
                    }
                    raw_data = &raw_data[layout.row_pitch as usize..];
                }
            }

            res
        };

        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, data)
            .with_context(|| "input data vector not right")?;
        buffer
            .save(p)
            .with_context(|| format!("failed to save to {p:?}"))
    }
}

pub fn get_current_time_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    return duration_since_epoch.as_secs_f64();
}

pub fn get_angle_f32(rate: f32) -> f32 {
    (crate::get_current_time_f64() * rate as f64).rem_euclid(2.0 * std::f64::consts::PI) as f32
}
