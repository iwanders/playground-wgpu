use glam::{Mat4, Vec2, Vec3, Vec3A, Vec4, vec3};
use wgpu::util::DeviceExt as _;
use zerocopy::IntoBytes as _;

/// A representation of a mesh on the CPU side.
#[derive(Clone)]
pub struct CpuMesh {
    /// The vertex position.
    pub position: Vec<Vec3>,

    /// The vertex indices
    pub index: Vec<u32>,

    //--- Optionals below.
    /// Name to use for the vertex buffer
    pub name: Option<String>,

    /// The vertex colors, do they have alpha..?
    pub color: Option<Vec<Vec4>>,

    /// The vertex normals
    pub normal: Option<Vec<Vec3A>>,

    /// The UV mapping
    pub uv: Option<Vec<Vec2>>,
}

impl CpuMesh {
    /// Create a new cpu mesh, not validating anything.
    pub fn new(position: Vec<Vec3>, index: Vec<u32>) -> Self {
        Self {
            position,
            index,
            color: None,
            normal: None,
            uv: None,
            name: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    fn get_name_prefix(&self) -> String {
        let name_prefix = if let Some(name) = self.name.as_ref() {
            format!("{}", name)
        } else {
            format!(
                "mesh_position_len_{}_index_len_{}",
                self.position.len(),
                self.index.len()
            )
        };
        name_prefix
    }

    /// Create a GPU mesh, the cpu mesh can go out of scope.
    pub fn to_gpu(&self, context: &crate::Context, layout: &wgpu::BindGroupLayout) -> GpuMesh {
        let name_prefix = self.get_name_prefix();

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_vertex", name_prefix)),
                contents: self.position.as_bytes(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_index", name_prefix)),
                contents: self.index.as_bytes(),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            });
        let index_length = self.index.len() as u32;

        let normal_data = self.normal.as_ref().map(|z| z.as_bytes()).unwrap_or(&[]);
        let normal_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_normal", name_prefix)),
                contents: normal_data,
                usage: wgpu::BufferUsages::STORAGE,
            });

        let color_data = self.color.as_ref().map(|z| z.as_bytes()).unwrap_or(&[]);
        let color_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_color", name_prefix)),
                contents: color_data,
                usage: wgpu::BufferUsages::STORAGE,
            });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: GpuMesh::MESH_BINDING_NORMAL,
                        resource: normal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: GpuMesh::MESH_BINDING_COLOR,
                        resource: color_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{}_bind_group", name_prefix)),
            });

        GpuMesh {
            vertex_buffer,
            index_buffer,
            index_length,
            normal_buffer,
            color_buffer,
            bind_group,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GpuMesh {
    pub bind_group: wgpu::BindGroup,

    /// Buffer holding the position data.
    pub vertex_buffer: wgpu::Buffer,
    /// Buffer holding the indices for the vertex data.
    pub index_buffer: wgpu::Buffer,
    /// TOtal number of indices.
    pub index_length: u32,

    //--- Optionals below, if they are unused, they are zero length, but still bound.
    pub normal_buffer: wgpu::Buffer,
    pub color_buffer: wgpu::Buffer,
}

impl GpuMesh {
    pub const MESH_BINDING_NORMAL: u32 = 0;
    pub const MESH_BINDING_COLOR: u32 = 1;
    pub const MESH_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh_layout"),
            entries: &[
                // Normals
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_BINDING_NORMAL,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Color data
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_BINDING_COLOR,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };

    /// The vertex attributes only contain the positions, they are the only required component.
    pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x3 ];
    /// Obtain the vertex layout.
    pub const fn get_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }
}
crate::static_assert_size!(Vec3, 12);
