use super::mesh::GpuMesh;
use crate::context::Context;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt as _;
use zerocopy::{Immutable, IntoBytes};

use crate::wgpu_util::StaticWgslStack;
const USE_WGSL_SHADER: bool = true;
pub const MESH_OBJECT_SLANG: &str = include_str!("mesh_object.slang");
pub const MESH_OBJECT_SPIRV: &[u8] = include_bytes!("mesh_object.spv");
pub const MESH_OBJECT_WGSL: StaticWgslStack = StaticWgslStack {
    name: "mesh_object",
    entry: "main",
    sources: &[
        include_str!("../shader_common.wgsl"),
        include_str!("mesh_object.wgsl"),
    ],
};

/// Something that owns a gpu mesh and generates vertices from it at the vertex stage
#[derive(Clone, Debug)]
pub struct MeshObject {
    /// The context we operate on.
    pub context: Context,

    /// Cpu representation of the instances.
    pub instances: Vec<Mat4>,

    /// Gpu representation of the instances.
    pub instances_buffer: wgpu::Buffer,

    /// The buffer for our uniform.
    pub mesh_object_uniform: wgpu::Buffer,

    /// The GPU mesh to operate on.
    pub gpu_mesh: GpuMesh,

    /// The bindgroup that contains all the buffers.
    pub bind_group: wgpu::BindGroup,
}

#[derive(Debug, Copy, Clone, PartialEq, IntoBytes, Immutable, Default)]
#[repr(C)]
pub struct MeshObjectMetaUniform {
    pub color_present: u32,
    pub normal_present: u32,
    pub uv_present: u32,
}

impl MeshObject {
    /// This creates a new mesh object with a dummy placeholder for instances.
    pub fn new(context: Context, gpu_mesh: GpuMesh) -> Self {
        let instances_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&gpu_mesh.name),
            size: std::mem::size_of::<Mat4>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let instances = vec![];
        let mesh_object_uniform = MeshObjectMetaUniform {
            color_present: gpu_mesh.color_present as u32,
            normal_present: gpu_mesh.normal_present as u32,
            uv_present: gpu_mesh.uv_present as u32,
        };
        let mesh_object_uniform =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{}_mesh_object_uniform", gpu_mesh.name)),
                    contents: mesh_object_uniform.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        let layout = context.device.create_bind_group_layout(&Self::MESH_LAYOUT);
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_OBJECT_UNIFORM_BINDING,
                        resource: mesh_object_uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_OBJECT_INSTANCES_BINDING,
                        resource: instances_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_BINDING_NORMAL,
                        resource: gpu_mesh.normal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_BINDING_COLOR,
                        resource: gpu_mesh.color_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_BINDING_UV,
                        resource: gpu_mesh.uv_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{}_bind_group", gpu_mesh.name)),
            });

        Self {
            context,
            instances,
            instances_buffer,
            mesh_object_uniform,
            gpu_mesh,
            bind_group,
        }
    }

    /// Set the object to have a single transform, does NOT update the gpu data.
    pub fn set_single_transform(&mut self, transform: &Mat4) {
        self.instances.resize(1, Default::default());
        self.instances[0] = *transform;
    }

    /// Set the object to have a this set of transforms, does NOT update the gpu data.
    pub fn set_transforms(&mut self, transform: &[Mat4]) {
        self.instances = transform.iter().copied().collect();
    }

    /// This replaces the current gpu data with a fresh buffer that holds the updated instance values.
    pub fn replace_gpu_data(&mut self) {
        let instances_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&self.gpu_mesh.name),
                    contents: self.instances.as_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        self.instances_buffer = instances_buffer;

        let layout = self
            .context
            .device
            .create_bind_group_layout(&Self::MESH_LAYOUT);
        let bind_group = self
            .context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_OBJECT_UNIFORM_BINDING,
                        resource: self.mesh_object_uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_OBJECT_INSTANCES_BINDING,
                        resource: self.instances_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_BINDING_NORMAL,
                        resource: self.gpu_mesh.normal_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_BINDING_COLOR,
                        resource: self.gpu_mesh.color_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: Self::MESH_BINDING_UV,
                        resource: self.gpu_mesh.uv_buffer.as_entire_binding(),
                    },
                ],
                label: Some(&format!("{}_bind_group", self.gpu_mesh.name)),
            });
        self.bind_group = bind_group;
    }

    pub fn add_commands(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.push_debug_group(&self.gpu_mesh.name);

        render_pass.set_bind_group(Self::MESH_OBJECT_SET, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.gpu_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.gpu_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(
            0..self.gpu_mesh.index_length,
            0,
            0..self.instances.len() as u32,
        );
        render_pass.pop_debug_group();
    }

    pub fn retrieve_embedded_shader(device: &wgpu::Device) -> super::VertexCreaterShader {
        if USE_WGSL_SHADER {
            super::VertexCreaterShader::new(MESH_OBJECT_WGSL.create(device), MESH_OBJECT_WGSL.entry)
        } else {
            let config = wgpu::ShaderModuleDescriptorPassthrough {
                label: Some("mesh_object.spv"),
                // spirv: None,
                spirv: Some(wgpu::util::make_spirv_raw(MESH_OBJECT_SPIRV)),
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
            super::VertexCreaterShader::new(
                unsafe { device.create_shader_module_passthrough(config) },
                "main",
            )
        }
    }

    pub const MESH_OBJECT_SET: u32 = 2;
    pub const MESH_OBJECT_UNIFORM_BINDING: u32 = 0;
    pub const MESH_OBJECT_INSTANCES_BINDING: u32 = 1;
    pub const MESH_BINDING_NORMAL: u32 = 2;
    pub const MESH_BINDING_COLOR: u32 = 3;
    pub const MESH_BINDING_UV: u32 = 4;
    pub const MESH_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh_object_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_OBJECT_UNIFORM_BINDING,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_OBJECT_INSTANCES_BINDING,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Normals
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_BINDING_NORMAL,
                    visibility: wgpu::ShaderStages::VERTEX,
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
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MESH_BINDING_UV,
                    visibility: wgpu::ShaderStages::VERTEX,
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

crate::static_assert_size!(Mat4, 4 * 4 * 4);

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_mesh_object_struct_align() {
        let module = MESH_OBJECT_WGSL.to_module();
        crate::verify_wgsl_struct_sized!(
            MeshObjectMetaUniform,
            module,
            color_present,
            normal_present,
            uv_present
        );
    }
}
