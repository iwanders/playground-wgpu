// I'm tired of struct sizes  / field alignment / values being off. So this here holds the grand checker logic
// that relies on a compute pipeline that on the slang side outputs the alignments such that we can check them
// against the rust side.
//
// Overengineered? Absolutely.
#[cfg(test)]
mod test {
    use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, TryFromBytes};

    pub const SLANG_SIZE_CHECK_SPIRV: &[u8] = include_bytes!("slang_size_check.spv");

    pub fn retrieve_embedded_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
        let config = wgpu::ShaderModuleDescriptorPassthrough {
            label: Some("slang_size_check.spv"),
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

    #[derive(Debug, Clone)]
    pub struct FieldInfo {
        pub struct_name: String,
        pub struct_size: u32,
        pub field_size: u32,
        pub field_offset: u32,
        pub field_name: String,
    }

    pub fn size_check_retrieve_struct_field_info(
        struct_name: &str,
        field_name: &str,
    ) -> Result<FieldInfo, anyhow::Error> {
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

        // Note no undercore between the two fields.
        let entry_point = format!("introspecter_{}{}", struct_name, field_name);
        // println!("entry_point: {entry_point:?}");
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &cs_module,
            entry_point: Some(&entry_point),
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

        // println!("trying to retrieve, we map the red aback bbuffer");
        let slice = read_back_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        // println!("And then read the value into a cpu vector.");
        let mut data = Vec::new();
        let slice = read_back_buffer.slice(..);
        let mapped = slice.get_mapped_range();
        data.extend_from_slice(&mapped);
        drop(mapped);

        // result[0] = sizeof(A);\
        // result[1] = sizeof(our_instance.B);\
        // result[2] = uint64_t(uint64_t(&our_instance.B) - uint64_t(&our_instance));\

        #[derive(IntoBytes, Immutable, FromBytes, KnownLayout, Debug, Clone, Copy)]
        #[repr(C, packed)]
        struct ResponseData {
            struct_size: u32,
            field_size: u32,
            field_offset: u32,
        }

        let (resp, _) = ResponseData::try_read_from_prefix(&data)
            .map_err(|_| anyhow::format_err!("ref_from_bytes failed"))?;

        Ok(FieldInfo {
            struct_name: struct_name.to_owned(),
            struct_size: resp.struct_size,
            field_size: resp.field_size,
            field_offset: resp.field_offset,
            field_name: field_name.to_owned(),
        })
    }

    fn get_size_of_return_type<F, T, U>(_f: F) -> usize
    where
        F: FnOnce(T) -> U,
    {
        std::mem::size_of::<U>()
    }
    macro_rules! check_struct {
        ($ty:ty, $( $e:ident ),*) => {
            $(
                let field_info = size_check_retrieve_struct_field_info(stringify!($ty), stringify!($e)).unwrap();
                assert_eq!(field_info.struct_name, stringify!($ty));
                assert_eq!(field_info.field_name, stringify!($e));
                assert_eq!(
                    field_info.field_offset,
                    std::mem::offset_of!($ty, $e) as u32
                );
                assert_eq!(
                    field_info.struct_size,
                    std::mem::size_of::<$ty>() as u32
                );
                assert_eq!(
                    field_info.field_size,
                    get_size_of_return_type((|s: $ty| s.$e)) as u32
                );
            )*
        };
    }

    #[test]
    fn test_view_uniform_size() {
        use crate::view::ViewUniform;
        check_struct!(ViewUniform, view_proj, camera_world_position);
    }
}
