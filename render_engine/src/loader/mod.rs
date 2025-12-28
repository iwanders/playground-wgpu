use crate::vertex::mesh::CpuMesh;
use glam::{Vec2, Vec3, Vec3A, Vec4, vec2, vec3, vec3a, vec4};

pub fn load_gltf(
    document: gltf::Document,
    buffers: &[gltf::buffer::Data],
    desired_index: usize,
) -> CpuMesh {
    let mut vertex_buffer = Vec::<Vec3>::new();
    let mut index_buffer: Vec<u32> = Vec::new();
    let mut normal_buffer: Option<Vec<Vec3A>> = None;
    let mut uv_buffer: Option<Vec<Vec2>> = None;
    let mut color_buffer: Option<Vec<Vec4>> = None;
    let mut name: Option<String> = None;
    for scene in document.scenes() {
        for (node_index, node) in scene.nodes().enumerate() {
            if node_index != desired_index {
                continue;
            }
            if let Some(mesh) = node.mesh() {
                name = mesh.name().map(|z| z.to_owned());
                for (mesh_index, primitive) in mesh.primitives().enumerate() {
                    let _ = mesh_index;
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    // Access vertex positions
                    if let Some(positions) = reader.read_positions() {
                        for p in positions {
                            vertex_buffer.push(vec3(p[0], p[1], p[2]));
                        }
                    }
                    // Access indices
                    if let Some(indices) = reader.read_indices() {
                        match indices {
                            ::gltf::mesh::util::ReadIndices::U8(iter) => {
                                index_buffer.extend(iter.map(|v| v as u32));
                            }
                            ::gltf::mesh::util::ReadIndices::U16(iter) => {
                                index_buffer.extend(iter.map(|v| v as u32));
                            }
                            ::gltf::mesh::util::ReadIndices::U32(iter) => {
                                index_buffer.extend(iter);
                            }
                        }
                    }

                    // Access colors
                    if let Some(colors) = reader.read_colors(0) {
                        let color_container = color_buffer.get_or_insert_default();
                        match colors {
                            gltf::mesh::util::ReadColors::RgbU8(_iter) => todo!(),
                            gltf::mesh::util::ReadColors::RgbU16(_iter) => todo!(),
                            gltf::mesh::util::ReadColors::RgbF32(iter) => {
                                color_container.extend(iter.map(|v| vec4(v[0], v[1], v[2], 1.0)));
                            }
                            gltf::mesh::util::ReadColors::RgbaU8(_iter) => todo!(),
                            gltf::mesh::util::ReadColors::RgbaU16(_iter) => todo!(),
                            gltf::mesh::util::ReadColors::RgbaF32(iter) => {
                                color_container.extend(iter.map(|v| vec4(v[0], v[1], v[2], v[3])));
                            }
                        }
                    }
                    // Access normals
                    if let Some(normals) = reader.read_normals() {
                        let normal_container = normal_buffer.get_or_insert_default();
                        // normal_container.resize(vertex_buffer.len(), Default::default());

                        for n in normals {
                            // Do something with the normal [n[0], n[1], n[2]]
                            normal_container.push(vec3a(n[0], n[1], n[2]));
                            // normal_container[ni] = vec3(n[0], n[1], n[2]);
                            // println!("normal: {:?}", normal_container[ni]);
                        }
                    }
                    // Access texture coordinates (TexCoords)
                    if let Some(tex_coords) = reader.read_tex_coords(0) {
                        let texture_container = uv_buffer.get_or_insert_default();
                        for tc in tex_coords.into_f32() {
                            texture_container.push(vec2(tc[0], tc[1]));
                        }
                    }
                }
            }
        }
    }
    let mut res = CpuMesh::new(vertex_buffer, index_buffer);
    res.color = color_buffer;
    res.normal = normal_buffer;
    res.uv = uv_buffer;
    res.name = name;
    res
}
