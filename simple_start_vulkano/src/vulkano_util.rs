use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::{
    Validated,
    buffer::{AllocateBufferError, Buffer, BufferCreateInfo, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator},
};
pub trait BufferHelpers {
    fn from_bytes(
        allocator: Arc<dyn MemoryAllocator>,
        create_info: BufferCreateInfo,
        allocation_info: AllocationCreateInfo,
        data: &[u8],
    ) -> Result<Subbuffer<[u8]>, Validated<AllocateBufferError>>;
    fn from_slice<T: BufferContents + Copy>(
        allocator: Arc<dyn MemoryAllocator>,
        create_info: BufferCreateInfo,
        allocation_info: AllocationCreateInfo,
        data: &[T],
    ) -> Result<Subbuffer<[T]>, Validated<AllocateBufferError>>;
}

impl BufferHelpers for vulkano::buffer::Buffer {
    fn from_bytes(
        allocator: Arc<dyn MemoryAllocator>,
        create_info: BufferCreateInfo,
        allocation_info: AllocationCreateInfo,
        data: &[u8],
    ) -> Result<Subbuffer<[u8]>, Validated<AllocateBufferError>> {
        let buffer =
            Buffer::new_slice::<u8>(allocator, create_info, allocation_info, data.len() as u64)?;

        {
            let mut write_guard = buffer.write().unwrap();

            for (o, i) in write_guard.iter_mut().zip(data.iter()) {
                *o = *i;
            }
        }
        Ok(buffer)
    }
    fn from_slice<T: BufferContents + Copy>(
        allocator: Arc<dyn MemoryAllocator>,
        create_info: BufferCreateInfo,
        allocation_info: AllocationCreateInfo,
        data: &[T],
    ) -> Result<Subbuffer<[T]>, Validated<AllocateBufferError>> {
        let buffer =
            Buffer::new_slice::<T>(allocator, create_info, allocation_info, data.len() as u64)?;

        {
            let mut write_guard = buffer.write().unwrap();

            for (o, i) in write_guard.iter_mut().zip(data.iter()) {
                *o = *i;
            }
        }
        Ok(buffer)
    }
}
