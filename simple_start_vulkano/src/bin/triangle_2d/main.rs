use std::io::Cursor;
use std::sync::Arc;

use anyhow::Context;
use glam::{Mat4, Vec3, Vec3A, Vec4, vec3, vec3a, vec4};
use log::*;
use simple_start::State;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CopyImageToBufferInfo};
use vulkano::command_buffer::{ClearColorImageInfo, CommandBufferUsage};
use vulkano::format::ClearColorValue;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::PipelineBindPoint;
use vulkano::sync::{self, GpuFuture};

use zerocopy::IntoBytes;
use zerocopy_derive::Immutable;

use vulkano::buffer::BufferContents;
#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable, BufferContents)]
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

struct LocalState(pub State);

impl std::ops::Deref for LocalState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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

        // THis is... something, but I can't directly pass a [Vertex;16] type thing? :/
        let buffer = Buffer::from_data::<[u8; 16 * std::mem::size_of::<Vertex>()]>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.as_bytes().try_into().unwrap(),
        )
        .unwrap();

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            self.device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            self.device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
                ..ClearColorImageInfo::image(self.image.clone())
            })
            .unwrap();

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
