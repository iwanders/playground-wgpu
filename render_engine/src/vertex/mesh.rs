use glam::{Vec2, Vec3, Vec3A, Vec4, vec3, vec4};
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
    // apparently some PBR materials have two UV maps :/
    /// Tangents, in mikktspace.
    pub tangents: Option<Vec<Vec4>>,
}

impl CpuMesh {
    /// Create a new cpu mesh, not validating anything!
    pub fn new(position: Vec<Vec3>, index: Vec<u32>) -> Self {
        Self {
            position,
            index,
            color: None,
            normal: None,
            uv: None,
            name: None,
            tangents: None,
        }
    }

    pub fn axis_frame() -> Self {
        fn make_axis_polys(axis: usize, alternate_axis: isize) -> [Vec3; 3] {
            let mut r = [
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 0.0),
            ];
            r[1][axis] = 1.0;
            if alternate_axis <= 0 {
                // Modulo to allow expressing -0 and 0 with 4 and -4
                r[0][alternate_axis.abs() as usize % 3] = 0.1;
            } else {
                r[2][alternate_axis as usize % 3] = 0.1;
            }
            r
        }
        let vertices: [([Vec3; 3], Vec4); _] = [
            // x axis, red
            (make_axis_polys(0, 1), vec4(1.0, 0.0, 0.0, 1.0)),
            // x axis back, red
            (make_axis_polys(0, -1), vec4(1.0, 0.0, 0.0, 1.0)),
            // y axis, green
            (make_axis_polys(1, 2), vec4(0.0, 1.0, 0.0, 1.0)),
            (make_axis_polys(1, -2), vec4(0.0, 1.0, 0.0, 1.0)),
            // z axis, blue, 4 to work around -0 and 0 being identical for integers.
            (make_axis_polys(2, 3), vec4(0.0, 0.0, 1.0, 1.0)),
            (make_axis_polys(2, -3), vec4(0.0, 0.0, 1.0, 1.0)),
        ];

        let position: Vec<Vec3> = vertices
            .iter()
            .map(|(p, _c)| p.iter())
            .flatten()
            .copied()
            .collect();
        let colors = vertices
            .iter()
            .map(|(_p, c)| [c, c, c])
            .flatten()
            .copied()
            .collect();

        let mut axis_mesh = Self {
            index: (0..position.len() as u32).collect(),
            position,
            color: Some(colors),
            normal: None,
            uv: None,
            name: Some("coordinate_frame".to_owned()),
            tangents: None,
        };
        axis_mesh.calculate_normals();
        axis_mesh
    }

    /// Set the useful name for renderdoc and diagnostic errors.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn calculate_tangents(&mut self) -> bool {
        // This needs to access the uv and normals, but it doesn't provide the API to propage it missing, so shield
        // against the invalid unwrap here.
        if self.uv.is_none() || self.normal.is_none() {
            return false;
        }
        self.tangents = Some(vec![Default::default(); self.position.len()]);
        bevy_mikktspace::generate_tangents(self)
    }

    pub fn calculate_normals(&mut self) {
        let mut normals: Vec<Vec3A> = vec![Default::default(); self.position.len()];
        for poly_indices in self.index.chunks(3) {
            let a = self.position[poly_indices[0] as usize];
            let b = self.position[poly_indices[1] as usize];
            let c = self.position[poly_indices[2] as usize];
            let this_normal = (b - a).cross(c - a); // Outward or inward normal? :/
            let this_normal = this_normal.normalize();
            normals[poly_indices[0] as usize] = this_normal.into();
            normals[poly_indices[1] as usize] = this_normal.into();
            normals[poly_indices[2] as usize] = this_normal.into();
        }
        self.normal = Some(normals);
    }

    pub fn get_name_prefix(&self) -> String {
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

    /// Create the gpu mesh, which is independent of the CPU Mesh.
    pub fn to_gpu(&self, context: &crate::Context) -> GpuMesh {
        // https://www.w3.org/TR/webgpu/#minimum-buffer-binding-size

        let name_prefix = self.get_name_prefix();

        // We can locally create this, and use it for the layout, it doesn't need to be the exact same instance.
        let layout = context
            .device
            .create_bind_group_layout(&crate::vertex::mesh::GpuMesh::MESH_LAYOUT);

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

        // No normals = no lighting... if there are no normals, build some normals.
        // Should we ensure all meshes just always have normals?

        let normal_data = self
            .normal
            .as_ref()
            .map(|z| z.as_bytes())
            .unwrap_or([Vec3A::ZERO].as_bytes());
        let normal_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_normal", name_prefix)),
                contents: normal_data,
                usage: wgpu::BufferUsages::STORAGE,
            });

        let color_data = self
            .color
            .as_ref()
            .map(|z| z.as_bytes())
            .unwrap_or([Vec3A::ZERO].as_bytes());
        let color_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_color", name_prefix)),
                contents: color_data,
                usage: wgpu::BufferUsages::STORAGE,
            });

        let uv_data = self
            .uv
            .as_ref()
            .map(|z| z.as_bytes())
            .unwrap_or([Vec2::ZERO].as_bytes());
        let uv_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_uv", name_prefix)),
                contents: uv_data,
                usage: wgpu::BufferUsages::STORAGE,
            });

        let tangent_data = self
            .tangents
            .as_ref()
            .map(|z| z.as_bytes())
            .unwrap_or([Vec4::ZERO].as_bytes());
        let tangent_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_tangent", name_prefix)),
                contents: tangent_data,
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
                    wgpu::BindGroupEntry {
                        binding: GpuMesh::MESH_BINDING_UV,
                        resource: uv_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{}_bind_group", name_prefix)),
            });

        GpuMesh {
            name: name_prefix,
            vertex_buffer,
            index_buffer,
            index_length,
            normal_buffer,
            normal_present: self.normal.is_some(),
            color_buffer,
            color_present: self.color.is_some(),
            uv_buffer,
            uv_present: self.uv.is_some(),
            tangent_buffer,
            tangent_present: self.tangents.is_some(),
            bind_group,
        }
    }
}

/// Representation of a mesh on the gpu side, not tied to the CpuMesh.
#[derive(Clone, Debug)]
pub struct GpuMesh {
    pub name: String,
    /// The bindgroup that contains all the buffers.
    pub bind_group: wgpu::BindGroup,

    /// Buffer holding the position data.
    pub vertex_buffer: wgpu::Buffer,
    /// Buffer holding the indices for the vertex data.
    pub index_buffer: wgpu::Buffer,
    /// Total number of indices.
    pub index_length: u32,

    //--- Optionals below, if they are unused, they are zero length, but still bound.
    pub normal_buffer: wgpu::Buffer,
    pub normal_present: bool,
    pub color_buffer: wgpu::Buffer,
    pub color_present: bool,

    pub uv_buffer: wgpu::Buffer,
    pub uv_present: bool,

    pub tangent_buffer: wgpu::Buffer,
    pub tangent_present: bool,
}

impl GpuMesh {
    pub const MESH_BINDING_NORMAL: u32 = 0;
    pub const MESH_BINDING_COLOR: u32 = 1;
    pub const MESH_BINDING_UV: u32 = 2;
    pub const MESH_BINDING_TANGENT: u32 = 3;
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
                // UV data
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_BINDING_UV,
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
crate::static_assert_size!(Vec3, GpuMesh::get_vertex_layout().array_stride as _);
