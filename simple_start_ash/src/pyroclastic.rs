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
// There's 21 methods in device that start with destroy_*  so... only 21 object types, that's a lot, but doable to wrap
// all and ensure proper destruction of them all by keeping the tree, for convenience we'll add derefs to all.
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
macro_rules! new_impl {
    ($type:ty,$target:ident, $target_type:ty) => {
        impl $type {
            pub fn new($target: $target_type) -> Self {
                Self {
                    $target: $target.into(),
                }
            }
        }
    };
}

macro_rules! new_with_parent_impl {
    ($type:ty,$parent_name:ident,$parent_type: ty, $path:ident,  $target:ty  ) => {
        impl $type {
            pub fn new($parent_name: $parent_type, $path: $target) -> Self {
                Self {
                    $parent_name: $parent_name.into(),
                    $path: $path.into(),
                }
            }
        }
    };
}

pub struct Instance {
    pub instance: Mutex<ash::Instance>,
}
deref_impl!(Instance, Mutex<ash::Instance>, instance);
new_impl!(Instance, instance, ash::Instance);

pub struct InnerDevice {
    pub instance: Arc<Instance>,
    pub device: Mutex<ash::Device>,
    pub pdevice: Mutex<ash::vk::PhysicalDevice>,
}
deref_impl!(InnerDevice, Mutex<ash::Device>, device);

// Gah, I need an enable_shared_from_this
#[derive(Clone)]
pub struct Device {
    inner: Arc<InnerDevice>,
}
deref_impl!(Device, InnerDevice, inner);

impl Device {
    pub fn new(
        instance: Arc<Instance>,
        device: ash::Device,
        pdevice: ash::vk::PhysicalDevice,
    ) -> Self {
        Self {
            inner: InnerDevice {
                instance,
                device: device.into(),
                pdevice: pdevice.into(),
            }
            .into(),
        }
    }
    pub fn get_physical_device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        let instance = self.instance.lock();
        let pdevice = self.pdevice.lock();

        unsafe { instance.get_physical_device_memory_properties(*pdevice) }
    }

    pub fn create_image_tracked(
        &self,
        img_info: &vk::ImageCreateInfo<'_>,
    ) -> Result<Image, vk::Result> {
        let image = unsafe { self.device.lock().create_image(img_info, None)? };
        Ok(Image::new(self.inner.clone(), image))
    }
    pub fn allocate_memory_tracked(
        &self,
        memory_info: &vk::MemoryAllocateInfo,
    ) -> Result<DeviceMemory, vk::Result> {
        let memory = unsafe { self.device.lock().allocate_memory(memory_info, None)? };
        Ok(DeviceMemory::new(self.inner.clone(), memory))
    }

    pub fn create_image_owned(
        &self,
        img_info: &vk::ImageCreateInfo,
        subresource: &vk::ImageSubresourceRange,
        memory_flags: vk::MemoryPropertyFlags,
    ) -> Result<ImageBuf, anyhow::Error> {
        let device_memory_properties = self.get_physical_device_memory_properties();

        // let instance = self.instance.lock();
        let image = self.create_image_tracked(img_info)?;

        let image_memory_req = unsafe {
            let device = self.device.lock();
            device.get_image_memory_requirements(*image)
        };
        let image_memory_index = find_memorytype_index(
            &image_memory_req,
            &device_memory_properties,
            memory_flags, // !!! Allocate it as host visible, and coherent.
        )
        .with_context(|| "Unable to find suitable memory index for image.")?;

        let image_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let image_memory = self.allocate_memory_tracked(&image_allocate_info)?;
        // let image_memory = unsafe { device.allocate_memory(&image_allocate_info, None)? };
        unsafe {
            let device = self.device.lock();
            device.bind_image_memory(*image, *image_memory, 0)?
        };

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

    pub fn get_device_queue_tracked(&self, queue_family_index: u32, queue_index: u32) -> Queue {
        let queue = unsafe {
            self.device
                .lock()
                .get_device_queue(queue_family_index, queue_index)
        };
        InnerQueue::new(self.inner.clone(), queue).into()
    }

    pub fn create_fence_tracked(
        &self,
        fence_create_info: &vk::FenceCreateInfo<'_>,
    ) -> Result<Fence, vk::Result> {
        let raw_fence = unsafe { self.device.lock().create_fence(&fence_create_info, None)? };
        Ok(Fence::new(self.inner.clone(), raw_fence))
    }
    pub fn create_semaphore_tracked(
        &self,
        semaphore_create_info: &vk::SemaphoreCreateInfo<'_>,
    ) -> Result<Semaphore, vk::Result> {
        let raw_semaphore = unsafe {
            self.device
                .lock()
                .create_semaphore(&semaphore_create_info, None)?
        };
        Ok(Semaphore::new(self.inner.clone(), raw_semaphore))
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
    pub fn finish(mut self, command_buffer: &vk::CommandBuffer) -> ash::prelude::VkResult<()> {
        self.finished = true;
        unsafe { self.end_command_buffer(*command_buffer) }
    }

    pub fn imagebuf_layout_barrier(
        &self,
        command_buffer: &vk::CommandBuffer,
        image: &ImageBuf,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        // Source image layout.
        let image_barrier = vk::ImageMemoryBarrier::default()
            .image(*image.image)
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

pub struct InnerQueue {
    pub device: Arc<InnerDevice>,
    pub queue: Mutex<vk::Queue>,
}
deref_impl!(InnerQueue, Mutex<vk::Queue>, queue);
new_with_parent_impl!(InnerQueue, device, Arc<InnerDevice>, queue, vk::Queue);

#[derive(Clone)]
pub struct Queue {
    inner: Arc<InnerQueue>,
}
deref_impl!(Queue, InnerQueue, inner);
impl From<InnerQueue> for Queue {
    fn from(value: InnerQueue) -> Self {
        Queue {
            inner: value.into(),
        }
    }
}

impl Queue {
    pub fn create_command_pool_tracked(
        &self,
        pool_create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<CommandPool, vk::Result> {
        let pool = unsafe {
            self.device
                .lock()
                .create_command_pool(&pool_create_info, None)?
        };
        Ok(CommandPool::new(self.inner.clone(), pool.into()))
    }
}

/// Allocator to the command buffer, comes from the queue.
#[derive(Clone)]
pub struct CommandPool {
    pub queue: Arc<InnerQueue>,
    // Commandbuffers are not actually tied to the pool, the pool is a management resource, not a lifetime source.
    pub pool: vk::CommandPool,
}
deref_impl!(CommandPool, vk::CommandPool, pool);
new_with_parent_impl!(CommandPool, queue, Arc<InnerQueue>, pool, vk::CommandPool);
impl CommandPool {
    pub fn allocate_command_buffers_tracked(
        &self,
        info: &vk::CommandBufferAllocateInfo,
    ) -> Result<Vec<CommandBuffer>, vk::Result> {
        let mut allocate_info = *info;
        allocate_info = allocate_info.command_pool(self.pool);

        // Command buffers are never destroyed? They're tied to the pool, which is destroyed from the device.
        let mut command_buffers = unsafe {
            let device = self.queue.device.lock();
            device.allocate_command_buffers(&allocate_info)?
        };

        // Now we have unsafe command buffers, which we wrap.
        Ok(command_buffers
            .drain(..)
            .map(|b| CommandBuffer::new(self.queue.device.clone(), b))
            .collect())
    }
}

#[derive(Clone)]
pub struct CommandBuffer {
    // This is not actually tied to the pool, the pool is a management resource, not a lifetime source.
    pub device: Arc<InnerDevice>,
    pub buffer: vk::CommandBuffer,
}
new_with_parent_impl!(
    CommandBuffer,
    device,
    Arc<InnerDevice>,
    buffer,
    vk::CommandBuffer
);
impl std::ops::Deref for CommandBuffer {
    type Target = vk::CommandBuffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

pub struct Image {
    pub device: Arc<InnerDevice>,
    pub image: vk::Image,
}
deref_impl!(Image, vk::Image, image);
new_with_parent_impl!(Image, device, Arc<InnerDevice>, image, vk::Image);

pub struct DeviceMemory {
    pub device: Arc<InnerDevice>,
    pub memory: vk::DeviceMemory,
}
deref_impl!(DeviceMemory, vk::DeviceMemory, memory);
new_with_parent_impl!(
    DeviceMemory,
    device,
    Arc<InnerDevice>,
    memory,
    vk::DeviceMemory
);

pub struct ImageView {
    pub device: Arc<InnerDevice>,
    pub view: vk::ImageView,
}
deref_impl!(ImageView, vk::ImageView, view);
new_with_parent_impl!(ImageView, device, Arc<InnerDevice>, view, vk::ImageView);

pub struct Fence {
    pub device: Arc<InnerDevice>,
    pub fence: vk::Fence,
}
deref_impl!(Fence, vk::Fence, fence);
new_with_parent_impl!(Fence, device, Arc<InnerDevice>, fence, vk::Fence);

pub struct Semaphore {
    pub device: Arc<InnerDevice>,
    pub semaphore: vk::Semaphore,
}
deref_impl!(Semaphore, vk::Semaphore, semaphore);
new_with_parent_impl!(
    Semaphore,
    device,
    Arc<InnerDevice>,
    semaphore,
    vk::Semaphore
);

// --------------------- helpers

/// An full image with its buffer and subimage resource.
pub struct ImageBuf {
    pub image: Image,
    pub memory: DeviceMemory, // This parents to Device.
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
