// This holds some 'owned' versions of the wgpu structs.
// Not sure if this is actually... useful.

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
