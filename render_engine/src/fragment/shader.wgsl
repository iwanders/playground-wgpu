// This relies on camera & light uniform from common.

@binding(CAMERA_UNIFORM_BINDING) @group(CAMERA_UNIFORM_SET)
var<storage, read> camera_uniform : CameraUniformType;

@binding(LIGHT_UNIFORM_BINDING) @group(LIGHT_UNIFORM_SET)
var<storage, read> light_uniform : array<Light>;

@binding(TEXTURE_UNIFORM_BINDING_TEXTURE) @group(TEXTURE_UNIFORM_SET)
var texture : binding_array<texture_2d<f32>>;

@binding(TEXTURE_UNIFORM_BINDING_SAMPLER) @group(TEXTURE_UNIFORM_SET)
var texture_sampler : binding_array<sampler>;

@binding(TEXTURE_UNIFORM_META) @group(TEXTURE_UNIFORM_SET)
var<storage, read> texture_uniform : array<TextureUniform>;

@fragment
fn main( input : CommonVertexOutput) -> CommonFragmentOutput
{
    // double_sided: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_doublesided
    // MUST have normals swapped

    var output: CommonFragmentOutput;
    let texture_meta = texture_uniform[0];

    let global_color = vec3<f32>(1.0, 1.0, 1.0); // we currently don't have this but it exists in the gltf.
    let vertex_color = input.color;

    var current_color = global_color * vertex_color;

    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_pbrmetallicroughness_basecolortexture
    // Base color texture is RGB, encoded with sRGB transfer function... What do we do with that?
    if ( texture_meta.base_color != 0){
        current_color *= (textureSample(texture[texture_meta.base_color], texture_sampler[texture_meta.base_color], input.uv_pos)).xyz;
    }

    // Should read these two globals, defaults are https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material-pbrmetallicroughness
    var metallic_factor = 1.0; // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_pbrmetallicroughness_metallicfactor
    var roughness_factor = 1.0; // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_pbrmetallicroughness_roughnessfactor

    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_pbrmetallicroughness_metallicroughnesstexture
    // metalness is from the b channel, rougnness from g, values are linear, red is ignored.
    if (texture_meta.metallic_roughness != 0){
        let metallic_sampled = (textureSample(texture[texture_meta.metallic_roughness], texture_sampler[texture_meta.metallic_roughness], input.uv_pos));
        roughness_factor *= metallic_sampled.g;
        metallic_factor *= metallic_sampled.b;
    }


    // On occlusion: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_occlusiontexture
    // linearly sampled from the red channel, other channels igenored. Higher values indicate that receive full indirect
    // lighting, lower values inidicate no direct lighting.
    var occlusion = 1.0;
    if (texture_meta.occlusion != 0){
        let occlusion_sample = (textureSample(texture[texture_meta.occlusion], texture_sampler[texture_meta.occlusion], input.uv_pos)).xyz;
        occlusion = occlusion_sample.r;
    }

    // Global factor.
    let emissive_factor = 1.0; // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_emissivefactor
    // Emissive: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_emissivetexture
    // controls the color and intensity of the light being emitted by the material, rgb components encoded with
    // sRGB, fourth component Alpha must be ignored.
    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_emissivefactor defaults to 0...
    // let emissive_factor = vec3f(0.0, 0.0, 0.0); // Lets ignore that for now, this way we assume it is 1.0 if there is
    // an emissive texture.
    var emission = vec3f(0.0, 0.0, 0.0);
    if (texture_meta.emissive != 0){
        emission = (textureSample(texture[texture_meta.emissive], texture_sampler[texture_meta.emissive], input.uv_pos)).rgb;
    }

    // On normals: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#_material_normaltexture
    // normal vectors use convention +x is right, +y is up, +z is towards the viewer, alpha must be ignored.
    // How do we reconcile this with the vertex normals?
    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-material-normaltextureinfo
    let normal_scale = 1.0; // of course, also a global.
    // Lets start with the normal from the vertices.
    var normal = normalize(input.normal);
    // And then in the schema for the texture info; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#schema-reference-material-normaltextureinfo
    // scaledNormal =  normalize((<sampled normal texture value> * 2.0 - 1.0) * vec3(<normal scale>, <normal scale>, 1.0))

    if (texture_meta.normal != 0){
        // Normal map only has red and green, but the formula does use it like rgba... lets do that.
        let normal_sampled = (textureSample(texture[texture_meta.normal], texture_sampler[texture_meta.normal], input.uv_pos)).rgb;
        // Does this multiply with the normal from the vertices? Or overwrite? Probably multiply?
        // I think this needs some more work... This needs some reading.
        normal *= ((normal_sampled * 2.0 - 1.0) * vec3f(normal_scale, normal_scale, 1.0));
    }


    // Next, we should do math to actually calculate lights... yeah this needs a LOT of work

   	let light_count : u32 = arrayLength(&light_uniform);

   	let view_vector = normalize(input.view_vector);


   	var color = vec3<f32>(0.0);
   	for (var i: u32 = 0; i < light_count; i++) {
   	    var this_light  = light_uniform[i];
   	    let light_type = this_light.light_type;
  		if (light_type == LIGHT_TYPE_OFF){
  		    continue;
  		}
        // Obtain light properties.
  		let light_direction = Light_direction(&this_light, input.world_pos);
  		let light_color = this_light.color;
  		let light_intensity = this_light.intensity; // should be scaled by distance.

        // Ehh super not right here. lol.
        var kd = roughness_factor;
        var ks = metallic_factor;

        var specular = 0.0;
        let lambertian = max(dot(light_direction, normal), 0.0);
        if lambertian > 0.0 {
            let view_direction = normalize(input.view_vector);
            let half_dir = normalize(light_direction + view_direction);
            let spec_angle = max(dot(half_dir, normal), 0.0);
            specular = pow(spec_angle, 16.0); // from wikipedia...
        }


  		// let kd = light_uniform.lights[i].hardness_kd_ks.y; // diffuse effect
  		// let ks = light_uniform.lights[i].hardness_kd_ks.z; // specular effect
  		// let R = reflect(light_direction, normal); // equivalent to 2.0 * dot(N, L) * N - L

  		let diffuse =  max(0.0, dot(light_direction, normal)) * light_color ;

  		// We clamp the dot product to 0 when it is negative
  		// let RoV = min(max(0.0, dot(R, view_vector)), 0.5);
  		// let specular = pow(RoV, hardness);
        // let specular = 0.1;

  		color += current_color * (kd * diffuse + ks * specular);
   	}

    color *= occlusion;

    color += emission;

   	let corrected_color= color;
    output.color = vec4<f32>(corrected_color, 1.0);
    return output;
}
