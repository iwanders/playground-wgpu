// This holds some 'owned' versions of the wgpu structs.
// Not sure if this is actually... useful... many of them aren't actually used.

pub struct StaticWgslStack {
    pub name: &'static str,
    pub entry: &'static str,
    pub sources: &'static [&'static str],
}
impl StaticWgslStack {
    pub fn create(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        let combined_string = self.sources.iter().cloned().collect::<String>();
        let descriptor = wgpu::ShaderModuleDescriptor {
            label: Some(self.name),
            source: wgpu::ShaderSource::Wgsl(combined_string.as_str().into()),
        };
        device.create_shader_module(descriptor)
    }
    pub fn to_module(&self) -> wgpu::naga::Module {
        let combined_string = self.sources.iter().cloned().collect::<String>();
        let module = naga::front::wgsl::parse_str(&combined_string).unwrap();
        module
    }
}

#[macro_export]
macro_rules! verify_field {
    ($Container:ty, $field:expr, $members:expr) => {
        let mut found: bool = false;
        for member in $members.iter() {
            let name = std::stringify!($field);
            if member.name.as_ref().map(|v| v.as_str()) == Some(name) {
                let rust_offset = std::mem::offset_of!($Container, $field) as u32;
                // Verify the offset.
                assert_eq!(
                    member.offset, rust_offset,
                    "offset of member {} does not match rust; {}, wgsl: {}",
                    name, rust_offset, member.offset
                );
                found = true;
            }
        }
        if !found {
            assert!(false, "could not find member {}", std::stringify!($field));
        }
    };
}
#[macro_export]
macro_rules! verify_wgsl_struct_sized {
    ($ty:ty, $shader_module:expr, $( $e:ident ),*) => {
        let our_struct_type = $shader_module
            .types
            .iter()
            .find(|z| z.1.name.as_ref().map(|v| v.as_str()) == Some(stringify!($ty)))
            .unwrap();
        if let naga::ir::TypeInner::Struct { members, span } = &our_struct_type.1.inner {
            assert_eq!(
                std::mem::size_of::<$ty>() as u32,
                *span,
                "Rust struct size does not match expected wgsl length: {}",
                *span
            );

            $(
                crate::verify_field!($ty, $e, members);
            )*
        } else {
            panic!("Incorrect type found");
        }
    };
}

#[derive(Debug, Clone)]
pub struct BindGroupLayoutDescriptorOwned {
    pub label: Option<String>,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutDescriptorOwned {
    pub fn into_descriptor(&self) -> wgpu::BindGroupLayoutDescriptor<'_> {
        wgpu::BindGroupLayoutDescriptor {
            label: self.label.as_ref().map(|v| v.as_str()),
            entries: &self.entries,
        }
    }
}
impl<'a> std::convert::From<&'a BindGroupLayoutDescriptorOwned>
    for wgpu::BindGroupLayoutDescriptor<'a>
{
    fn from(item: &'a BindGroupLayoutDescriptorOwned) -> Self {
        item.into_descriptor()
    }
}

/*
#[derive(Debug, Clone)]
pub struct PipelineLayoutDescriptorOwned {
    pub label: Option<String>,
    pub bind_group_layouts: Vec<wgpu::BindGroupLayout>,
    pub immediate_size: u32,
}
// probably needs a two-tier conversion?

#[derive(Debug, Clone)]
pub struct PipelineCompilationOptionsOwned {
    pub constants: std::collections::HashMap<String, f64>,
    pub zero_initialize_workgroup_memory: bool,
}
#[derive(Debug, Clone)]
pub struct VertexBufferLayout {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

#[derive(Debug, Clone)]
pub struct VertexStateOwned {
    pub module: wgpu::ShaderModule,
    pub label: Option<String>,
    pub compilation_options: PipelineCompilationOptionsOwned,
    pub buffers: Vec<VertexBufferLayout>,
}

#[derive(Debug, Clone)]
pub struct FragmentStateOwned {
    pub module: wgpu::ShaderModule,
    pub label: Option<String>,
    pub compilation_options: PipelineCompilationOptionsOwned,
    pub targets: Vec<Option<wgpu::ColorTargetState>>,
}

#[derive(Debug, Clone)]
pub struct RenderPipelineDescriptorOwned {
    pub label: Option<String>,
    // pub layout: Option<&'a PipelineLayout>,
    pub vertex: VertexStateOwned,
    pub primitive: wgpu::PrimitiveState,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub multisample: wgpu::MultisampleState,
    pub fragment: Option<FragmentStateOwned>,
    pub multiview_mask: Option<std::num::NonZeroU32>,
    // pub cache: Option<&'a PipelineCache>,
}
*/
