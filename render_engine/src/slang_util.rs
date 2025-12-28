// I'm tired of struct sizes  / field alignment / values being off. So this here holds the grand checker logic
// that relies on a compute pipeline that on the slang side outputs the alignments such that we can check them
// against the rust side.
//
// Overengineered? Absolutely.

pub const SLANG_SIZE_CHECK_SPIRV: &[u8] = include_bytes!("slang_size_check.spv");

pub fn retrieve_embedded_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    let config = wgpu::ShaderModuleDescriptorPassthrough {
        label: Some("mesh_object.spv"),
        // spirv: None,
        spirv: Some(wgpu::util::make_spirv_raw(SLANG_SIZE_CHECK_SPIRV)),
        entry_point: "".to_owned(),
        // This is unused for SPIR-V
        num_workgroups: (0, 0, 0),
        runtime_checks: wgpu::ShaderRuntimeChecks::unchecked(),
        dxil: None,
        msl: None,
        hlsl: None,
        glsl: None,
        wgsl: None,
    };
    unsafe { device.create_shader_module_passthrough(config) }
}
#[cfg(test)]
mod test {
    use wgpu::util::DeviceExt as _;

    use super::*;

    #[test]
    fn test_reflection() {
        let crate::context::ContextReturn { context, target } =
            pollster::block_on(crate::Context::new_sized(1, 1)).unwrap();

        const BUFFER_SIZE: u64 = 1024;

        let device = &context.device;
        // Create a buffer
        let buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some("our output"),
            size: BUFFER_SIZE,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let read_back_buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some("our_read_back_buffer"),
            size: BUFFER_SIZE,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bgl_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Custom Storage Bind Group Layout"),
            entries: &[bgl_entry],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Combined Storage Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let cs_module = retrieve_embedded_shader(device);

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &cs_module,
            entry_point: Some("introspecter_CameraUniformcamera_world_position"),
            compilation_options: Default::default(),
            cache: None,
        });

        // now, we dispatch
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute pass descriptor"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&compute_pipeline);
            cpass.set_bind_group(0, Some(&bind_group), &[]);

            cpass.dispatch_workgroups(1, 1, 1);
        }

        // And copy from the storage buffer to the mappable buffer.
        encoder.copy_buffer_to_buffer(&buffer, 0, &read_back_buffer, 0, BUFFER_SIZE);

        context.queue.submit(Some(encoder.finish()));

        println!("trying to retrieve, we map the red aback bbuffer");
        let slice = read_back_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        println!("And then read the value into a cpu vector.");
        let mut data = Vec::new();
        let slice = read_back_buffer.slice(..);
        let mapped = slice.get_mapped_range();
        data.extend_from_slice(&mapped);
        drop(mapped);
        // buffer.unmap();

        println!("result: {data:?}");
    }
}
