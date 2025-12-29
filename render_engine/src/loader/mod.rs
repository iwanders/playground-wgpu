use crate::vertex::mesh::CpuMesh;
use crate::{fragment::mesh_object_textured::MeshObjectTextured, vertex::mesh_object::MeshObject};
use anyhow::Context as _;
use glam::{Mat4, Vec2, Vec3, Vec3A, Vec4, vec2, vec3, vec3a, vec4};

struct MeshIndex(usize);

fn load_gltf_meshes(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> Vec<(MeshIndex, CpuMesh)> {
    let mut result = vec![];
    for scene in document.scenes() {
        for (node_index, node) in scene.nodes().enumerate() {
            let _ = node_index;
            if let Some(mesh) = node.mesh() {
                let mut vertex_buffer = Vec::<Vec3>::new();
                let mut index_buffer: Vec<u32> = Vec::new();
                let mut normal_buffer: Option<Vec<Vec3A>> = None;
                let mut uv_buffer: Option<Vec<Vec2>> = None;
                let mut color_buffer: Option<Vec<Vec4>> = None;
                let name: Option<String>;

                let primitives: Vec<_> = mesh.primitives().collect();
                if primitives.len() > 1 {
                    todo!();
                }
                if primitives.is_empty() {
                    continue;
                }

                name = mesh.name().map(|z| z.to_owned());
                let primitive = primitives.first().unwrap();
                {
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

                    let mut this_mesh = CpuMesh::new(vertex_buffer, index_buffer);
                    this_mesh.color = color_buffer;
                    this_mesh.normal = normal_buffer;
                    this_mesh.uv = uv_buffer;
                    this_mesh.name = name;

                    result.push((MeshIndex(mesh.index()), this_mesh));
                }
            }
        }
    }
    result
}

pub fn load_gltf(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    desired_index: usize,
) -> CpuMesh {
    load_gltf_meshes(&document, buffers)[desired_index]
        .1
        .clone()
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

trait WrappingModeToWgpu {
    fn to_wgpu(&self) -> wgpu::AddressMode;
}
impl WrappingModeToWgpu for gltf::texture::WrappingMode {
    fn to_wgpu(&self) -> wgpu::AddressMode {
        match self {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        }
    }
}
trait MagFilterModeToWgpu {
    fn to_wgpu(&self) -> wgpu::FilterMode;
}
impl MagFilterModeToWgpu for gltf::texture::MagFilter {
    fn to_wgpu(&self) -> wgpu::FilterMode {
        match self {
            gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest,
            gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
        }
    }
}
trait MinFilterModeToWgpu {
    fn to_wgpu(&self) -> wgpu::FilterMode;
}
impl MinFilterModeToWgpu for gltf::texture::MinFilter {
    fn to_wgpu(&self) -> wgpu::FilterMode {
        match self {
            gltf::texture::MinFilter::Nearest => wgpu::FilterMode::Nearest,
            gltf::texture::MinFilter::Linear => wgpu::FilterMode::Linear,
            gltf::texture::MinFilter::NearestMipmapNearest => wgpu::FilterMode::Nearest,
            gltf::texture::MinFilter::LinearMipmapNearest => wgpu::FilterMode::Linear,
            gltf::texture::MinFilter::NearestMipmapLinear => wgpu::FilterMode::Linear,
            gltf::texture::MinFilter::LinearMipmapLinear => wgpu::FilterMode::Linear,
        }
    }
}

trait TransformToGlam {
    fn to_glam(&self) -> Mat4;
}
impl TransformToGlam for gltf::scene::Transform {
    fn to_glam(&self) -> Mat4 {
        match self {
            gltf::scene::Transform::Matrix { matrix } => {
                // is column major, so this works in one shot.
                Mat4::from_cols_array_2d(matrix)
            }
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                Mat4::from_rotation_translation(
                    glam::Quat::from_array(*rotation),
                    Vec3::from_array(*translation),
                ) * Mat4::from_scale(Vec3::from_array(*scale))
            }
        }
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

pub fn load_gltf_objects(
    context: &crate::Context,
    gltf_path: &std::path::Path,
) -> Result<Vec<MeshObjectTextured>, anyhow::Error> {
    let device = &context.device;
    let (document, buffers, images) = gltf::import(gltf_path)?;
    // This doesn't handle instancing nicely atm... but this is already a non-tested hour long bender.

    // Okay, so we have a sampler specification.
    // Textures then point to sampler specification.
    //
    // Nodes builds the scene tree-wise.
    //  Nodes have a mesh and potentially children.
    //  Nodes have a transform.
    //
    // Mesh has a name and a list of primitives.
    //   Primitive holds:
    //      Material, model, and mesh attributes (positions, normals, texcoords)
    //
    // Material has... a bunch of properties which we can't handle, but for the helmet they're 1.0...
    //   pbr_metallic_roughness:
    //       base color texture
    //       metallic roughness texture
    //       normal t
    //  normal texture
    //  occlusion texture
    //  emissive texture
    //   (emissive factor)
    //
    // Accessors, some buffers and buffer views, which I'm not sure how to deal with.

    // Okay, so not the end of the world.
    //  First: Obtain auxiliary data; collect the textures & samplers.
    //         Collect a bunch of raw meshes.
    //  Traverse the nodes to propagate the transforms and combine the textures with the meshes as MeshObjectTextured

    let meshes_by_index = load_gltf_meshes(&document, &buffers);

    // Load some textures... this doesn't actually get me the samplers.
    // let textures: Vec<wgpu::Texture> = images
    //     .iter()
    //     .map(|z| load_gltf_texture(&context, z))
    //     .collect();

    let mut textured_samplers = vec![];
    for texture in document.textures() {
        let sampler = texture.sampler();
        let image_source = texture.source();
        let image_data = gltf::image::Data::from_source(image_source.source(), None, &buffers)?;
        println!(
            "image data: {:?}, {:?}",
            image_data.format, image_data.width,
        );
        let texture = load_gltf_texture(&context, &image_data);
        textured_samplers.push(crate::texture::SampledTexture {
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: sampler.wrap_s().to_wgpu(),
                address_mode_v: sampler.wrap_t().to_wgpu(),
                address_mode_w: sampler.wrap_t().to_wgpu(), // no w in gltf?
                mag_filter: sampler
                    .mag_filter()
                    .unwrap_or(gltf::texture::MagFilter::Linear)
                    .to_wgpu(), // Nearest
                min_filter: sampler
                    .min_filter()
                    .unwrap_or(gltf::texture::MinFilter::Linear)
                    .to_wgpu(),
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            texture: texture,
            texture_type: crate::texture::TextureType::None,
        });
    }

    // Okay, so now we have the textures & samplers combined at the ready... now, we should be able to mostly iterate
    // through the nodes, fiddle a bit with transforms and when we reach a mesh with a material check its material and
    // then pick resources from the our textures that are now ready to go.

    let mut output = vec![];

    // Need to recursively traverse the tree of nodes, lets keep a stack to do so.
    #[derive(Debug, Copy, Clone)]
    struct Stack {
        transform: Mat4,
        index: usize,
    };
    let mut stack = std::collections::VecDeque::new();
    stack.push_back(Stack {
        transform: Mat4::IDENTITY,
        index: 0,
    });
    let flat_nodes = document.nodes().collect::<Vec<_>>();
    let flat_materials = document.materials().collect::<Vec<_>>();
    while let Some(top) = stack.pop_front() {
        let this_node = &flat_nodes[top.index];

        // Apply this nodes' transform.
        let this_transform = this_node.transform().to_glam() * top.transform;

        if let Some(mesh) = this_node.mesh() {
            // Okay we have a mesh... now we need to do actual hard work to build our desired output object.
            // Retrieve the mesh from our already processed entries.
            let actual_geometry = meshes_by_index
                .iter()
                .find(|z| z.0.0 == mesh.index())
                .with_context(|| "could not find mesh")?;
            // This cuts a corner, but we don't handle multiple primitives atm anyway.
            let primitives = mesh.primitives().collect::<Vec<_>>();
            if primitives.len() > 1 {
                todo!("we don't handle meshes with multiple primitives");
            }
            if let Some(this_primitive) = primitives.first() {
                let mut this_primitive_textures = vec![];
                // Now, we do something with the material
                if let Some(material_index) = this_primitive.material().index() {
                    let this_material = &flat_materials[material_index];
                    // let double_sided = this_material.double_sided();
                    let alpha_mode = this_material.alpha_mode();
                    if alpha_mode != gltf::material::AlphaMode::Opaque {
                        todo!();
                    }

                    if let Some(emissive_texture) = this_material.emissive_texture() {
                        let texture_index = emissive_texture.texture().index();
                        let mut with_sampler = textured_samplers[texture_index].clone();
                        with_sampler.texture_type = crate::texture::TextureType::Emissive;
                        this_primitive_textures.push(with_sampler);
                    }

                    if let Some(base_color_texture) =
                        this_material.pbr_metallic_roughness().base_color_texture()
                    {
                        let texture_index = base_color_texture.texture().index();
                        let mut with_sampler = textured_samplers[texture_index].clone();
                        with_sampler.texture_type = crate::texture::TextureType::BaseColor;
                        this_primitive_textures.push(with_sampler);
                    }

                    if let Some(metallic_roughness_texture) = this_material
                        .pbr_metallic_roughness()
                        .metallic_roughness_texture()
                    {
                        let texture_index = metallic_roughness_texture.texture().index();
                        let mut with_sampler = textured_samplers[texture_index].clone();
                        with_sampler.texture_type = crate::texture::TextureType::MetallicRoughness;
                        this_primitive_textures.push(with_sampler);
                    }
                    // let metallic_factor = this_material.pbr_metallic_roughness().metallic_factor();
                    // let base_color_factor =
                    //     this_material.pbr_metallic_roughness().base_color_factor();
                    // let roughness_factor =
                    //     this_material.pbr_metallic_roughness().roughness_factor();
                    //

                    if let Some(normal_texture) = this_material.normal_texture() {
                        let texture_index = normal_texture.texture().index();
                        let mut with_sampler = textured_samplers[texture_index].clone();
                        with_sampler.texture_type = crate::texture::TextureType::Normal;
                        this_primitive_textures.push(with_sampler);
                    }
                    if let Some(occlusion_texture) = this_material.occlusion_texture() {
                        let texture_index = occlusion_texture.texture().index();
                        let mut with_sampler = textured_samplers[texture_index].clone();
                        with_sampler.texture_type = crate::texture::TextureType::Occlusion;
                        this_primitive_textures.push(with_sampler);
                    }
                }

                // Now that we have processed the material, we have obtained the textures... we can instantiate our
                // desired MeshObjectTextured.
                let gpu_mesh = actual_geometry.1.to_gpu(&context);
                let mut mesh_object = MeshObject::new(context.clone(), gpu_mesh);
                mesh_object.set_single_transform(&this_transform);
                mesh_object.replace_gpu_data();

                output.push(MeshObjectTextured::new(
                    context.clone(),
                    mesh_object,
                    &this_primitive_textures,
                ));
            }
        }

        for c in this_node.children() {
            stack.push_back(Stack {
                transform: this_transform,
                index: c.index(),
            });
        }
    }

    println!("element count: {:?}", output.len());

    Ok(output)
}
