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
//  Dynamic rendering does away with the render passess.
//    https://github.com/KhronosGroup/Vulkan-Samples/blob/97fcdeecf2db26a78b432b285af3869a65bb00bd/samples/extensions/dynamic_rendering/
// Don't support swapchain resizing.
// Understand vkPresentMode
// do not rely on vkQueueWaitIdle, it does not eliminate pipelining.
//  oh; https://youtu.be/QtBKLnpxzAw
//  we currently have some awful hybrid, where we do parts with dynamic rendering and parts with the pipeline?
//
// Vulkanised 2025: So You Want to Write a Vulkan Renderer in 2025 - Charles Giessen
//      https://www.youtube.com/watch?v=7CtjMfDdTdg
// buffer device address; use pointers without descriptors.
// use scalar block layout, mirrors the C layout.
// timeline semaphores, monotonically increasing u64, incremented when work is done, can wait on it to be value.
//  https://docs.vulkan.org/refpages/latest/refpages/source/vkWaitSemaphores.html
//  https://github.com/KhronosGroup/Vulkan-Samples/tree/97fcdeecf2db26a78b432b285af3869a65bb00bd/samples/extensions/timeline_semaphore
//
// From A Gentle Introduction to Vulkan for Rendering and Compute Workloads - Vulkan Course
//  https://youtu.be/nD83r06b5NE?t=4435
//  https://docs.vulkan.org/refpages/latest/refpages/source/VK_EXT_shader_object.html
//     VK_EXT_shader_object does away with the pipeline object?
//     https://www.khronos.org/blog/you-can-use-vulkan-without-pipelines-today
//       no raytracing :< but we can mix and match according to this document.
//     https://github.com/KhronosGroup/Vulkan-Docs/blob/main/proposals/VK_EXT_shader_object.adoc
//       no multiview
//     https://docs.vulkan.org/spec/latest/chapters/shaders.html#shaders-objects-state
//     https://github.com/KhronosGroup/Vulkan-Samples/tree/97fcdeecf2db26a78b432b285af3869a65bb00bd/samples/extensions/shader_object
//
//
// Some good diagrams here: https://github.com/David-DiGioia/vulkan-diagrams
//
// https://developer.nvidia.com/vulkan-memory-management
// Oh, and on the reverse z buffer; https://developer.nvidia.com/blog/visualizing-depth-precision/
//
// Multiple objects? https://docs.vulkan.org/tutorial/latest/16_Multiple_Objects.html
//
//  Hmm, looks like command buffers are persistent... so they can be reused and submitted again?
//
//
// What's VK_EXT_descriptor_indexing ? from https://amini-allight.org/post/vknew-modern-vulkan-with-descriptor-indexing-dynamic-rendering-and-shader-objects
//
// https://github.com/SaschaWillems/Vulkan
//
use anyhow::Context;
use ash::ext::debug_utils;
use ash::{Entry, vk};
use log::*;
use std::ffi;
use std::path::Path;
use zerocopy::IntoBytes;

use std::rc::Rc;
use std::sync::Arc;

use parking_lot::Mutex;
// use std::marker::PhantomData;

// Okay, so ash is completely unsafe; https://github.com/ash-rs/ash/issues/665#issuecomment-2030659066
//
// The vulkan spec has many many statements of; Host access to * must be externally synchronized. Where * is anything
// from Instance, Device and Queue...
//
// On parents; https://github.com/KhronosGroup/Vulkan-Docs/blob/8ae4650710cc67941a4caf807ef23c76cdc97059/chapters/fundamentals.adoc#L311-L321

// Host access to instance must be externally synchronized
// https://docs.vulkan.org/spec/latest/chapters/initialization.html

// https://docs.vulkan.org/spec/latest/chapters/devsandqueues.html#VkDevice, Host access to device must be externally synchronized]
// Making 'safe' wrappers for everything is bad, because that means that things like Fence and DeviceMemory must have a pointer to the device
// itself to ensure correct destruction order... or we need to introduce lifetimes, but that is also kinda :/
//
// Many things are handles only. Lets just wrap the ones that make sense? Goal is a reasonable level of 'safety' and abstraction, but not
// strictly correct, since that's a very hard goal to achieve in a performant way?
//
// How do we clear this all up correctly... maybe that's where that gpu allocator comes in for memory at least? Since they
// are all tied to that?
//
// Maybe we just make something that is _mostly correct_?

/// A context that holds the key functionality to interact with the vulkan stuff?
pub struct Ctx {
    pub instance: Mutex<ash::Instance>,
    pub device: Mutex<ash::Device>,
    pub pdevice: Mutex<ash::vk::PhysicalDevice>,
}
impl Drop for Ctx {
    fn drop(&mut self) {
        let instance = self.instance.lock();
        let device = self.device.lock();
        let pdevice = self.pdevice.lock();
    }
}
pub type CtxPtr = Arc<Ctx>;
impl Ctx {
    pub fn get_physical_device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        let instance = self.instance.lock();
        let pdevice = self.pdevice.lock();

        unsafe { instance.get_physical_device_memory_properties(*pdevice) }
    }
    // Lets build a pipeline!}

    pub fn create_image_owned(
        &self,
        img_info: &vk::ImageCreateInfo,
        subresource: &vk::ImageSubresourceRange,
        memory_flags: vk::MemoryPropertyFlags,
    ) -> Result<ImageBuf, anyhow::Error> {
        let instance = self.instance.lock();
        let device = self.device.lock();
        let pdevice = self.pdevice.lock();
        let image = unsafe { device.create_image(img_info, None)? };
        info!("image: {image:?}");
        let memory_req = unsafe { device.get_image_memory_requirements(image) };
        info!("memory_req: {memory_req:#?}");

        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(*pdevice) };
        let image_memory_req = unsafe { device.get_image_memory_requirements(image) };
        let image_memory_index = find_memorytype_index(
            &image_memory_req,
            &device_memory_properties,
            memory_flags, // !!! Allocate it as host visible, and coherent.
        )
        .with_context(|| "Unable to find suitable memory index for image.")?;
        info!("image_memory_index: {image_memory_index:#?}");

        let image_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let image_memory = unsafe { device.allocate_memory(&image_allocate_info, None)? };
        unsafe { device.bind_image_memory(image, image_memory, 0)? };

        Ok(ImageBuf {
            image,
            subresource: *subresource,
            memory: image_memory,
        })
    }

    pub fn record_command_buffer<'a, 'b>(
        &'a self,

        buffer: &vk::CommandBuffer,
        info: &vk::CommandBufferBeginInfo,
    ) -> Result<CommandBufferWriter<'a>, vk::Result> {
        let device = self.device.lock();
        unsafe { device.begin_command_buffer(*buffer, &info)? };
        Ok(CommandBufferWriter {
            device,
            finished: false,
        })
    }
}

pub struct CommandBufferWriter<'a> {
    device: parking_lot::MutexGuard<'a, ash::Device>,
    finished: bool,
}
impl<'a> std::ops::Deref for CommandBufferWriter<'a> {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl<'a> Drop for CommandBufferWriter<'a> {
    fn drop(&mut self) {
        if !self.finished {
            // Print an error... because vk may segfault on the todo :grimace:.
            error!("you forgot to add .finish(cmd_buffer) on the writer");
            todo!("you forgot to add .finish(cmd_buffer) on the writer");
        }
    }
}

impl<'a> CommandBufferWriter<'a> {
    pub fn finish<'b, T>(mut self, buffer: T) -> ash::prelude::VkResult<()>
    where
        T: Into<&'b vk::CommandBuffer>,
    {
        self.finished = true;
        let buffer: &'b vk::CommandBuffer = buffer.into();
        unsafe { self.end_command_buffer(*buffer) }
    }

    pub fn imagebuf_layout_barrier<'b, T>(
        &self,
        command_buffer: T,
        image: &ImageBuf,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) where
        T: Into<&'b vk::CommandBuffer>,
    {
        let command_buffer: &'b vk::CommandBuffer = command_buffer.into();
        // Source image layout.
        let image_barrier = vk::ImageMemoryBarrier::default()
            .image(image.image)
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .subresource_range(image.subresource);

        unsafe {
            self.cmd_pipeline_barrier(
                *command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );
        }
    }
}

/// A queue to submit commands to, these are limited and may be shared between threads.
pub struct Queue {
    pub ctx: CtxPtr,
    pub queue: Mutex<vk::Queue>,
}

/// Allocator to the command buffer, comes from the queue.
pub struct CommandPool {
    pub queue: Arc<Queue>,
    pub pool: vk::CommandPool,
}

pub struct CommandBuffer {
    pub pool: Arc<CommandPool>,
    pub buffer: vk::CommandBuffer,
}
impl std::ops::Deref for CommandBuffer {
    type Target = vk::CommandBuffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

pub struct ImageBuf {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory, // This parents to Device.
    pub subresource: vk::ImageSubresourceRange,
}

pub struct State {
    pub ctx: Ctx,

    // pub instance: ash::Instance,
    // pub device: ash::Device,
    // pub pdevice: ash::vk::PhysicalDevice,
    pub queue: vk::Queue,
    pub pool: vk::CommandPool,
    pub setup_command_buffer: vk::CommandBuffer,
    pub draw_command_buffer: vk::CommandBuffer,
    pub image: ImageBuf,
    // pub image_memory: vk::DeviceMemory,
    pub depth_image: ImageBuf,
    // pub depth_image_memory: vk::DeviceMemory,
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
pub mod shader_util;

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
    pub fn new(width: u32, height: u32) -> anyhow::Result<State> {
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
                // ash::khr::buffer_device_address::NAME.as_ptr(),
                // ash::ext::shader_object::NAME.as_ptr()
                // ash::khr::dynamic_rendering_local_read::NAME.as_ptr(),
            ];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,

                ..Default::default()
            };
            let priorities = [0.0];

            let mut physical_device_features13 =
                vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);

            let mut physical_device_features12 =
                vk::PhysicalDeviceVulkan12Features::default().buffer_device_address(true);

            // let mut local_read_features =
            //     vk::PhysicalDeviceDynamicRenderingLocalReadFeaturesKHR::default()
            //         .dynamic_rendering_local_read(true);

            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);
            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .push_next(&mut physical_device_features12)//.push_next(&mut local_read_features)
                .push_next(&mut physical_device_features13)
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

        let ctx = Ctx {
            instance: instance.clone().into(),
            device: device.clone().into(),
            pdevice: pdevice.clone().into(),
        };

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
            ) // Flag as destination now
            .mip_levels(1)
            .array_layers(1)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .image_type(vk::ImageType::TYPE_2D);
        let subresource = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let image = ctx.create_image_owned(
            &img_info,
            &subresource,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_img_info = vk::ImageCreateInfo::default()
            .format(vk::Format::D32_SFLOAT)
            .extent(extent)
            .samples(vk::SampleCountFlags::TYPE_1)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
            )
            .mip_levels(1)
            .array_layers(1)
            .image_type(vk::ImageType::TYPE_2D);
        let depth_subresource = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        let depth_image = ctx.create_image_owned(
            &depth_img_info,
            &depth_subresource,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let draw_commands_reuse_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let setup_commands_reuse_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        let rendering_complete_semaphore =
            unsafe { device.create_semaphore(&semaphore_create_info, None)? };

        Ok(State {
            // instance,
            // device,
            // pdevice,
            ctx,
            width,
            height,
            queue,
            pool,
            setup_command_buffer,
            draw_command_buffer,
            image,
            // image_memory,
            depth_image,
            // depth_image_memory,
            draw_commands_reuse_fence,
            setup_commands_reuse_fence,
            rendering_complete_semaphore,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
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
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .tiling(vk::ImageTiling::LINEAR)
            .image_type(vk::ImageType::TYPE_2D);
        let subresource = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let image = self.ctx.create_image_owned(
            &img_info,
            &subresource,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        let writer = self
            .ctx
            .record_command_buffer(&self.draw_command_buffer, &command_buffer_begin_info)?;
        // Execute commands.
        unsafe {
            //
            // let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            //     .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            // writer.begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)?;

            writer.imagebuf_layout_barrier(
                &self.draw_command_buffer,
                &self.image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            );

            writer.imagebuf_layout_barrier(
                &self.draw_command_buffer,
                &image,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );

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
            writer.cmd_copy_image(
                self.draw_command_buffer,
                self.image.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );

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
        };

        let image_memory = image.image;
        // Then, we map the memory, and copy it...
        //
        let data = {
            let device = self.ctx.device.lock();
            let mut res = vec![];

            let subres = vk::ImageSubresource::default().aspect_mask(vk::ImageAspectFlags::COLOR);
            let layout = unsafe { device.get_image_subresource_layout(image.image, subres) };
            info!("layout: {layout:#?}");
            unsafe {
                let mut raw_location = device.map_memory(
                    image.memory,
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

//https://github.com/rust-lang/rust/blob/4f14395c37db4c1be874e6b0ace6721674223c22/compiler/rustc_index/src/lib.rs#L36
#[macro_export]
macro_rules! static_assert_size {
    ($ty:ty, $size:expr) => {
        const _: [(); $size] = [(); ::std::mem::size_of::<$ty>()];
    };
}
