use ash::{Entry, vk};
use log::*;
use simple_start::State;
use zerocopy_derive::{Immutable, IntoBytes};

// https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/

struct LocalState(pub State);

impl std::ops::Deref for LocalState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, Immutable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.0, 0.0, 0.0],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.0, 0.0, 0.0],
    }, // E
];
const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

fn make_clear_rgba(r: u32, g: u32, b: u32, a: u32) -> vk::ClearColorValue {
    let mut res = vk::ClearColorValue::default();
    unsafe {
        res.uint32[0] = r;
        res.uint32[1] = g;
        res.uint32[2] = b;
        res.uint32[3] = a;
    }
    res
}

impl LocalState {
    pub async fn draw(&self) -> anyhow::Result<()> {
        unsafe {
            //
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device
                .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)?;

            let rendering_info =
                vk::RenderingInfo::default()
                    .layer_count(1)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: vk::Extent2D {
                            width: self.width,
                            height: self.height,
                        },
                    });
            self.device
                .cmd_begin_rendering(self.draw_command_buffer, &rendering_info);
            {
                // Source image layout, set to available for writing.
                let image_barrier = vk::ImageMemoryBarrier::default()
                    .image(self.image)
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
            self.device.cmd_clear_color_image(
                self.draw_command_buffer,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &make_clear_rgba(233, 23, 23, 255),
                &[vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }],
            );

            self.device.cmd_end_rendering(self.draw_command_buffer);
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
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Ok(())
    }
}

async fn async_main() -> std::result::Result<(), anyhow::Error> {
    let mut state = LocalState(State::new(256, 256).await?);
    state.draw().await?;
    state.save("/tmp/triangle_2d.png").await?;

    Ok(())
}

pub fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::builder()
        .is_test(false)
        .filter_level(log::LevelFilter::Info)
        // .filter_level(log::LevelFilter::max())
        .try_init()?;
    pollster::block_on(async_main())?;
    println!("Hello, world! ");
    Ok(())
}
