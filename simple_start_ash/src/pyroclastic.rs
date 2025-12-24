/// Like fast moving vulcanic ash.
///
/// Goal; Speed up development by providing helpers. Provide full access to everything.
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
//
// There's 21 methods in device that start with destroy_*  so... only 21 object types.
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

macro_rules! deref_impl {
    ($type:ty,$target:ty, $path:ident) => {
        impl std::ops::Deref for $type {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.$path
            }
        }
    };
}

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

        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(*pdevice) };
        let image_memory_req = unsafe { device.get_image_memory_requirements(image) };
        let image_memory_index = find_memorytype_index(
            &image_memory_req,
            &device_memory_properties,
            memory_flags, // !!! Allocate it as host visible, and coherent.
        )
        .with_context(|| "Unable to find suitable memory index for image.")?;

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

/// A queue to submit commands to, these are limited and may be shared between threads, so lets mutex it.
pub struct Queue {
    pub ctx: CtxPtr,
    pub queue: Mutex<vk::Queue>,
}

/// Allocator to the command buffer, comes from the queue.
pub struct CommandPool {
    pub queue: Arc<Queue>,
    pub pool: vk::CommandPool,
}
deref_impl!(CommandPool, vk::CommandPool, pool);

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

pub struct Image {
    pub ctx: CtxPtr,
    pub image: vk::Image,
}
deref_impl!(Image, vk::Image, image);

pub struct DeviceMemory {
    pub ctx: CtxPtr,
    pub memory: vk::DeviceMemory,
}
deref_impl!(DeviceMemory, vk::DeviceMemory, memory);

pub struct ImageView {
    pub ctx: CtxPtr,
    pub view: vk::ImageView,
}
deref_impl!(ImageView, vk::ImageView, view);

/// An full image with its buffer and subimage resource.
pub struct ImageBuf {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory, // This parents to Device.
    pub subresource: vk::ImageSubresourceRange,
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
