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
    pub queue: ash::vk::Queue,
    pub pool: ash::vk::CommandPool,
    pub setup_command_buffer: ash::vk::CommandBuffer,
    pub draw_command_buffer: ash::vk::CommandBuffer,
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

impl State {
    // ...
    pub async fn new(width: u32, height: u32) -> anyhow::Result<State> {
        let entry = Entry::linked();

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };
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
            extension_names.push(
                ffi::CStr::from_bytes_with_nul_unchecked(b"VK_EXT_headless_surface\0").as_ptr(),
            );
            extension_names
                .push(ffi::CStr::from_bytes_with_nul_unchecked(b"VK_EXT_debug_utils\0").as_ptr());
        }

        let appinfo = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 2, 0));

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
                "{:?}, {:?}",
                std::ffi::CStr::from_bytes_until_nul(props.device_name.as_bytes()),
                props.device_type
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
            let device_extension_names_raw = [ash::khr::swapchain::NAME.as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);
            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);
            instance
                .create_device(pdevice, &device_create_info, None)
                .unwrap()
        };
        info!("device created");
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let headless_instance = ash::ext::headless_surface::Instance::new(&entry, &instance);
        dbg!();

        let surface = unsafe {
            let create_info = ash::vk::HeadlessSurfaceCreateInfoEXT {
                flags: ash::vk::HeadlessSurfaceCreateFlagsEXT::empty(),
                ..Default::default()
            };
            headless_instance.create_headless_surface(&create_info, None)?
        };
        info!("surface: {surface:?}");
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
        let img_info = vk::ImageCreateInfo::default()
            .format(vk::Format::R8G8B8A8_UNORM)
            .image_type(vk::ImageType::TYPE_2D);
        let image = unsafe { device.create_image(&img_info, None)? };

        Ok(State {
            instance,
            device,
            width,
            height,
            queue,
            pool,
            setup_command_buffer,
            draw_command_buffer,
        })
    }

    pub async fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let p: &Path = path.as_ref();
        todo!();
        // let buffer_slice = self.buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        // let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        // buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        //     tx.send(result).unwrap();
        // });
        // self.device
        //     .poll(wgpu::PollType::wait_indefinitely())
        //     .unwrap();
        // rx.receive().await.unwrap().unwrap();

        // let data = buffer_slice.get_mapped_range();

        // use image::{ImageBuffer, Rgba};
        // let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, data).unwrap();
        // buffer
        //     .save(p)
        //     .with_context(|| format!("failed to save to {p:?}"))
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
