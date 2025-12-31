
// Random notes:
//   Probably does much better with image based lighting (IBL) which we currently don't have.
//   Even https://github.khronos.org/glTF-Sample-Viewer-Release/?model=https://raw.GithubUserContent.com/KhronosGroup/glTF-Sample-Assets/main/./Models/DamagedHelmet/glTF-Binary/DamagedHelmet.glb
//   with IBL disabled and only point lights looks kinda... 'meh', but it only has one light from the looks of it.
//   In some sources omega is used to depict vectors (w_i for incidence, w_o for out?) in the equation form.
//
// "Background: Physics and Math of Shading": by Naty Hoffman
//      https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf
//      Excellent read to understand this all, notes below for myself:
//
//  p7: Metals do not have subusrface scattering.
//  p9:
//      L: Radiance,
//      L_i: radiance incoming to the surface
//      L_o outgoing radiance
//  p9/10:
//      \mathbf{v}: unit length vector pointing along outgoing (iw: so towards the camera)
//      \mathbf{l}: unit length vector pointing opposite of incoming direction (so from surface to light)
//      BRDF: f(l,v)
//      \mathbf{n}: surface normal vector.
//      \mathbf{t} is the tangent vector defining a preferred direction over the surface (for anisotropic materials)
//
//  p11; lots of good explanation
//      BRDF is only defined for light & view above the surface, so dot(n,l) and dot(n,v) must both be non-negative.
//      We just multiply RGB valued BRDF by RGB valued light colors.
//  p12:
//      reciprocity; BRDF is the same if l and v are swapped.
//      energy conservation: surface cannot reflect more than all the light.
//      BRDF is composed of two physical phenomena;
//          surface reflection  (specular)
//          subsurface scattering (diffuse)
//      Surface reflectance, specular:
//          Based on microfacet theory.
//          Introduce \mathbf{m} microgeometry normal, this is the orientation of an individual microfacet.
// p13
//          Only if \mathbf{m} is halfway between l and v does it actually reflect from l to v.
//          \mathbf{h}: Half-vector, or half-angle vector
//          Omitting the derivation, the specular BRDF form is proposed (4)
//
//          f_{ufacet}(l,v) = \frac{F(l,h) G(l,v,h) D(h)}{4(n*l)(n*v)}   (4)
//
//          D(h) is the microgeometry Normal Distribution Function (NDF) evaluation at h, effectively the ratio ufacet that can reflect.
//          G(l,v,h) is the geometry function, also called geometry factor, shadow-masking function, shadowing function, geometry term.
// p14
//          D * G: concentration of active surface points that participate in reflecting light from l into v.
//          F(l,h) is the Fresnel reflectance, how much of the incoming light is reflected.
//          Divisor 4(n*l)(n*v) is a correction factor from local microgeomtry to overall macro.
//
//          Fresnel reflectance
//              Full equations are complex, and not convenient for artists.
// p15
//              F_{Schlick}(F_0, l, n) = F_0 + (1 - F_0) * (1 - (l * n))^5    (5)
// p16
//              Note this is a different 'n', so we substitute it for the active micogeometry normal h:
//
//              F_{Schlick}(F_0, l, h) = F_0 + (1 - F_0) * (1 - (l * h))^5    (6)
//              Also, note about you should use linear space for shading.
// p17
//              Table with F_0s, and explanation that F_0 is either very low, or very high. Most materials 2-5%.
// p18
//          Normal Distribution Function
//              Most surfaces do not have a uniform distribution of microgeometry fragments. Most face up towards \mathbf{n}.
//              Value is NOT restricted to [0.0, 1.0], positive, can be arbitrarily large, and a scalar.
//              D() determines the size, brightness and shape of the specular highlight.
//              Plethora of other sources cited for further reading.
//          Geometry function
//              Determines the probability that surface point with microgeometry normal m is visible by both l and v.
//              G(h) is a probability, so [0.0, 1.0].
//              Usually an approximation. Either uses no parameters, or uses the roughness parameters also used by D.
//              Takes care of the energy conservation.
//
// p20
//      Subsurface Reflecance (Diffuse Term)
//
//          Lambertian model is simple and often used, constant value as (n*l) is part of reflectance, not BRDF.
//          f_{Lambert}(l, v) = \frac{c_{diff}}{\pi}    (7)
//       With c_diff RGB value in [0.0, 1.0], 'surface color', actually called the diffuse color.
//
// p21
//      Other terms
//          missing subsurface scattering, multiple-bounce surface reflectance... etc
//
//      Implementing Physical Shading Models for Production
//
//          Needs an illumination model.
//          General Lighting; correctly sampling from all other objects...
// p22
//          Image-Based Lighting
//          Area light sources: shading BRDFs is easier than IBL.
// p23
//          Punctual Light sources
//              Common in games; point, directional & spot.
//              Specified by two quantities; light color c_light, and light direction vector l_c.
//              A derivation to show the integral disappears in favor of a single evaluation of the BRDF.
// p24
//              Common to clamp the dot product to zero to avoid back-facing light contributions
//              c_light falls off, inverse-square, but often other falloff is used.
//              Multiple light sources, eq 13 is calculated for each and summed.
//          Ambient lighting
//              Often only applied to diffuse lighting term.
//              Sometimes also applied to specular term, see 11,27,64
//
//      Building a Physically Based Shading Model
//          Effectively, boils down to chosing D, G... most papers introduce one of those.
//  p25..
//      Compares various functions for D with each other;
//          Phong NDF
//          Beckmann Distribution
//          Trowbridge-Reitz (GGX Distribution?)
// p28
//          Comparison shows that many materials are not well-modeled by any of these models, so three more are discussed.
//          ABC (Low et al.)
//          Bagher et all; Shifted Gamma Distribution SGD.
// p32
//          Burley; Generalized-Twobridge-Reitz
// p33
//          What to recomment, Phong NDF is simplest, reasonably expressive. Otherwise Trowbridge-Reitz is a good fit.
//          If more expressive, two-paramter; use GTR; Generalized Trowbridge-Reitz from Burley.
//
// p34
//      Choosing a Geometry function
//          Often too dark.
//          Cook-Torrance, not  affected by roughness.
//          Recommends Smith family of geometric functions.
//
//
//
//
// Which links to [35]; https://renderwonk.com/publications/s2010-shading-course/
//  https://renderwonk.com/publications/s2010-shading-course/hoffman/s2010_physically_based_shading_hoffman_b.pdf
//  https://renderwonk.com/publications/s2010-shading-course/hoffman/s2010_physically_based_shading_hoffman_b_notes.pdf
//
// Also helpful; https://boksajak.github.io/blog/BRDF has a nice pdf with further explanation.
//
// Okay, so this (the actual lighting) is _actually_ not rocket science, there's just a lot of symbols and variations in notation.
//
// Roughly; outgoing light = self_emitted_light + \Sum_{lights}(brdf_sampled * incoming_light * fresnel_part)
//
// Where brdf_sampled is created with that whole microfacet scattering model, effectively handling direct reflections and diffuse 'reflections'.
// The fresnel part makes things reflect more at shallower angles.
// The incoming light is just the attenuated light from the source.
// Self emitted light is... well self emitted light, no one seems to talk about this, but it seems simple enough to
// get something reasonable.
//
// GLTF on material structure:
//      https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#material-structure
//
// GLTF also has a section on microfacet surfaces:
//  https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#microfacet-surfaces
// Oh, and an sample implementation; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#implementation
// linear to srgb; https://github.com/KhronosGroup/glTF/issues/697#issuecomment-257186564 ? hmm..?
//
//
// Normals are another problem:
//  We need to rotate the normal map with the mesh position.
//  See https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/lighting-and-material/normal-mapping.html
//
// GLTF:
//  https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#foreword:~:text=When%20tangents%20are%20not%20specified%2C%20client%20implementations%20SHOULD%20calculate%20tangents
//  When tangents are not specified, client implementations SHOULD calculate tangents using default MikkTSpace algorithms
//  with the specified vertex positions, normals, and texture coordinates associated with the normal texture.
//
//
//
//
// Okay, so just rotating the normals & normal mapping is like... super complex, as described on http://www.mikktspace.com/
// Reference implementation; https://github.com/mmikk/MikkTSpace
// The Bevy folks conveniently ported this to a standalone rust implementation.
//
// And then we follow GLTF's viewer for the actual calculation guidance, since we're using GLTF models.
// Vertex shader:
// https://github.com/KhronosGroup/glTF-Sample-Renderer/blob/e6b052db89fb2adbaf31da4565a08265c96c2b9f/source/Renderer/shaders/primitive.vert#L135-L148
// Fragment shader:
// https://github.com/KhronosGroup/glTF-Sample-Renderer/blob/e6b052db89fb2adbaf31da4565a08265c96c2b9f/source/Renderer/shaders/material_info.glsl#L172-L175
// ------------------------------------------------------------------------------------


// This relies on camera & light uniform from common.
@binding(CAMERA_UNIFORM_BINDING) @group(CAMERA_UNIFORM_SET)
var<storage, read> camera_uniform : CameraUniformType;

@binding(LIGHT_UNIFORM_BINDING) @group(LIGHT_UNIFORM_SET)
var<storage, read> light_uniform : array<Light>;

// And on textures & samplers.
@binding(TEXTURE_UNIFORM_BINDING_TEXTURE) @group(TEXTURE_UNIFORM_SET)
var texture : binding_array<texture_2d<f32>>;

@binding(TEXTURE_UNIFORM_BINDING_SAMPLER) @group(TEXTURE_UNIFORM_SET)
var texture_sampler : binding_array<sampler>;

// And on the texture uniform that specifies which texture is what.
@binding(TEXTURE_UNIFORM_META) @group(TEXTURE_UNIFORM_SET)
var<storage, read> texture_uniform : array<TextureUniform>;


/// If this is true, the mesh is shaded with the shading normal values.
const DEBUG_OUTPUT_NORMALS: bool = false;
/// And its conversion function.
fn normal_to_display_color(normal: vec3f) -> CommonFragmentOutput{
    var output: CommonFragmentOutput;
    output.color = vec4f(normal * 0.5 + 0.5, 1.0);
    return output;
}

fn vec3f_to_out(z: vec3f) -> CommonFragmentOutput{
    var output: CommonFragmentOutput;
    output.color = vec4f(z, 1.0);
    return output;
}

/// A helper struct to store data that goes into the actual pbr logic functions.
struct SurfaceLightParameters {
    half_dir: vec3f,
    light_dir: vec3f,
    view_dir: vec3f,
    light_color: vec3f,
    albedo: vec3f,
    light_intensity: f32,
    normal: vec3f,
    roughness_factor: f32,
    metallic_factor: f32,
    occlusion: f32,
};

/// GGX normal distribution function.
fn brdf_D_ggx(alpha: f32, nh: f32) -> f32 {
    let a2 = alpha * alpha;
    let hnh = heaviside(nh);
    let nominator = a2 * hnh;
    let nh2 = nh * nh;
    let denominator = PI_F * pow((nh2 * (a2 - 1.0) + 1.0), 2);
    return nominator / denominator;
}

fn brdf_G_smith_part(alpha: f32, normal_part: f32, half_part: f32) -> f32 {
    let a2 = alpha * alpha;
    let nominator = 2.0 * abs(normal_part) * heaviside(half_part);
    let denominator = abs(normal_part) + sqrt(a2 + (1.0 - a2)*pow(normal_part, 2));
    return nominator / denominator;
}

fn brdf_G_smith_joint_masking_shadowing(alpha: f32, half_dir: vec3f, normal: vec3f, view_dir: vec3f, light_dir: vec3f) -> f32 {
    // Made up of a left term and right term.
    // Left  term takes alpha, h, n, l,
    // Right term takes alpha, h, n, v
    let left = brdf_G_smith_part(alpha, dot(normal, light_dir), dot(normal, light_dir));
    let right = brdf_G_smith_part(alpha, dot(normal, view_dir), dot(normal, view_dir));
    return left * right;
}

fn schlick_fresnel(f0: vec3f, vh: f32) -> vec3f {
    let to_fifth = pow(1.0 - abs(vh), 5);
    return f0 + (1.0 - f0) * to_fifth;
}

fn SurfaceLightParameters_calculate(me: ptr<function, SurfaceLightParameters>) -> vec3<f32> {
    // Okay, so here we actually do the PBR things!
    // gltf's material model: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#complete-model
    // For now, we're only doing something with the metal parts of it.
    // There's an informative implementation section; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#implementation
    // So for D they use GGX
    // For G they use Smith joint masking-shadowing function...
    // Lambertian diffuse
    // Schlick Fresnel approximation.


    // We immediately go for the approach of
    // https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#metal-brdf-and-dielectric-brdf

    let params = (*me);


    // GLTF docs state alpha in their equations is roughness squared;
    let alpha = params.roughness_factor * params.roughness_factor;
    let nh = dot(params.normal, params.half_dir);
    let D = brdf_D_ggx(alpha, nh);
    let G = brdf_G_smith_joint_masking_shadowing(alpha, params.half_dir, params.normal, params.view_dir, params.light_dir);
    let V = G / (4.0 * abs(dot(params.normal, params.light_dir)) * abs(dot(params.normal, params.view_dir)));
    // This does not have the simplification... they propose in the docs.

    let specular_brdf = V * D;

    let color = params.albedo * params.occlusion;
    let c_diff = mix(color, vec3f(0.0, 0.0, 0.0), params.metallic_factor);
    let f0 = mix(vec3f(0.04, 0.04, 0.04), color, params.metallic_factor);


    let vh = dot(params.view_dir, params.half_dir);
    let F = schlick_fresnel(f0, vh);

    let diffuse_brdf =  (1.0 / PI_F) * c_diff;

    let f_diffuse = (1.0 - F) * diffuse_brdf  ;
    let f_specular = F * specular_brdf  ;
    let material = (f_diffuse + f_specular);
    return material * (params.light_color * params.light_intensity);

    /*
    let lambertian = max(dot(params.light_dir, params.normal), 0.0);
    if lambertian > 0.0 {
        let spec_angle = max(dot(params.half_dir, params.normal), 0.0);
        // specular = pow(spec_angle, 16.0); // from wikipedia...
    }

    // let kd = light_uniform.lights[i].hardness_kd_ks.y; // diffuse effect
    // let ks = light_uniform.lights[i].hardness_kd_ks.z; // specular effect
    // let R = reflect(light_direction, normal); // equivalent to 2.0 * dot(N, L) * N - L

    let diffuse =  max(0.0, dot(params.light_dir, params.normal)) * params.light_color * params.light_intensity;

    // We clamp the dot product to 0 when it is negative
    let R = reflect(-params.light_dir, params.normal);
    let RoV = max(0.0, dot(R, params.view_dir));
    let hardness = 20.0;
    let specular = pow(RoV, hardness);

    return params.albedo  * (kd * diffuse   + ks * specular);
    */
}

@fragment
fn main(input : CommonVertexOutput) -> CommonFragmentOutput
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
        let occlusion_strength = 1.0;
        occlusion = (1.0 + occlusion_strength * (occlusion_sample.r - 1.0));
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
    var normal : vec3f = normalize(input.normal);
    // And then in the schema for the texture info; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#schema-reference-material-normaltextureinfo
    // scaledNormal =  normalize((<sampled normal texture value> * 2.0 - 1.0) * vec3(<normal scale>, <normal scale>, 1.0))

    if (texture_meta.normal != 0){
        // See the section around normal mapping in the big comment above why we are doing this here. It's the mikktspace conversion.
        // after https://github.com/KhronosGroup/glTF-Sample-Renderer/blob/e6b052db89fb2adbaf31da4565a08265c96c2b9f/source/Renderer/shaders/material_info.glsl#L172-L175
        let normal_sampled = (textureSample(texture[texture_meta.normal], texture_sampler[texture_meta.normal], input.uv_pos)).rgb;
        let global_normal_scale = 1.0;
        let normal_scaled = (normal_sampled * 2.0 - vec3f(1.0)) * vec3f(global_normal_scale, global_normal_scale, 1.0);
        let normal_scaled_normalized = normalize(normal_scaled);

        // Finally, use the tangent, bitangent and normal that we created in the vertex schader:
        normal = normalize(mat3x3f(input.tangent_w, input.bitangent_w, input.normal_w) * normal_scaled_normalized);
    }

    if DEBUG_OUTPUT_NORMALS {
        return normal_to_display_color(normal);
    }
    // Whew, we now have working normals...

    let light_count : u32 = arrayLength(&light_uniform);

    // View vector is from the contact point towards the camera.
   	let view_vector = normalize(input.view_vector);

   	var color = vec3<f32>(0.0);
   	for (var i: u32 = 0; i < light_count; i++) {
   	    var this_light  = light_uniform[i];
   	    let light_type = this_light.light_type;
  		if (light_type == LIGHT_TYPE_OFF){
  		    continue;
  		}

        // Light direction is from input.world_pos towards the light.
  		let light_direction = Light_direction(&this_light, input.world_pos);
        let half_dir = normalize(light_direction + view_vector);

  		let light_color = this_light.color;
  		let light_intensity = Light_intensity(&this_light, input.world_pos);

        var surface_light_parameters: SurfaceLightParameters;
        surface_light_parameters.half_dir = half_dir;
        surface_light_parameters.light_dir = light_direction;
        surface_light_parameters.view_dir = view_vector;
        surface_light_parameters.light_color = light_color;
        surface_light_parameters.albedo = current_color;
        surface_light_parameters.light_intensity = light_intensity;
        surface_light_parameters.normal = normal;
        surface_light_parameters.roughness_factor = roughness_factor;
        surface_light_parameters.metallic_factor = metallic_factor;
        surface_light_parameters.occlusion = occlusion;


        color += SurfaceLightParameters_calculate(&surface_light_parameters);

   	}

    color *= occlusion;

    color += emission;

   	let corrected_color = color;
    // let corrected_color = pow(color, vec3f(2.2));
    output.color = vec4<f32>(corrected_color, 1.0);
    return output;
}
