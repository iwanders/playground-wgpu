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

fn gltf_to_rgba8unorm(image: &gltf::image::Data) -> image::RgbaImage {
    match &image.format {
        gltf::image::Format::R8 => todo!(),
        gltf::image::Format::R8G8 => todo!(),
        gltf::image::Format::R8G8B8 => {
            let orig = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                image.width,
                image.height,
                image.pixels.clone(),
            )
            .unwrap();
            image::DynamicImage::ImageRgb8(orig).to_rgba8()
        }
        gltf::image::Format::R8G8B8A8 => image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            image.width,
            image.height,
            image.pixels.clone(),
        )
        .unwrap(),
        gltf::image::Format::R16 => todo!(),
        gltf::image::Format::R16G16 => todo!(),
        gltf::image::Format::R16G16B16 => todo!(),
        gltf::image::Format::R16G16B16A16 => todo!(),
        gltf::image::Format::R32G32B32FLOAT => todo!(),
        gltf::image::Format::R32G32B32A32FLOAT => todo!(),
    }
}

pub fn load_gltf_texture(context: &crate::Context, image: &gltf::image::Data) -> wgpu::Texture {
    // Do we need to do any color space mapping?
    let rgba8_image = gltf_to_rgba8unorm(image);
    let texture_size = wgpu::Extent3d {
        width: rgba8_image.width(),
        height: rgba8_image.height(),
        depth_or_array_layers: 1,
    };
    // Lets just do a single mip level for now.
    let texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: None, // TODO
        view_formats: &[],
    });

    context.queue.write_texture(
        // Tells wgpu where to copy the pixel data
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        // The actual pixel data
        &rgba8_image.as_raw(),
        // The layout of the texture
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture_size.width), // 4 because we used rgba8
            rows_per_image: Some(texture_size.height),
        },
        texture_size,
    );
    context.queue.submit([]);
    texture
}
