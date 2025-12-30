
// Random notes:
//   Probably does much better with image based lighting (IBL) which we currently don't have.
//   Even https://github.khronos.org/glTF-Sample-Viewer-Release/?model=https://raw.GithubUserContent.com/KhronosGroup/glTF-Sample-Assets/main/./Models/DamagedHelmet/glTF-Binary/DamagedHelmet.glb
//   with IBL disabled and only point lights looks kinda... 'meh', but it only has one light from the looks of it.
//   In some sources omega is used to depict vectors (w_i for incidence, w_o for out?) in the equation form.
//
// GLTF on material structure:
//      https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#material-structure
//
// GLTF also has a section on microfacet surfaces:
//  https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#microfacet-surfaces
// Oh, and an sample implementation; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#implementation
//
//
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
// Also helpful; https://boksajak.github.io/blog/BRDF has a nice pdf
//
// Okay, so this is _actually_ not rocket science, there's just a lot of symbols and variations in notation.
//
// Roughly; outgoing light = self_emitted_light + \Sum_{lights}(brdf_sampled * incoming_light * fresnel_part)
//
// Where brdf_sampled is created with that whole microfacet scattering model, effectively handling direct reflections and diffuse 'reflections'.
// The fresnel part makes things reflect more at shallower angles.
// The incoming light is just the attenuated light from the source.
// Self emitted light is... well self emitted light, no one seems to talk about this, but it seems simple enough to
// get something reasonable.
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


// Output normals as the color of the mesh.
const DEBUG_OUTPUT_NORMALS: bool = true;
fn normal_to_display_color(normal: vec3f) -> CommonFragmentOutput{
    var output: CommonFragmentOutput;
    output.color = vec4f(normal * 0.5 + 0.5, 1.0);
    return output;
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
    var normal : vec3f = normalize(input.normal);
    // And then in the schema for the texture info; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#schema-reference-material-normaltextureinfo
    // scaledNormal =  normalize((<sampled normal texture value> * 2.0 - 1.0) * vec3(<normal scale>, <normal scale>, 1.0))

    if (texture_meta.normal != 0){
        // Normal map only has red and green, but the formula does use it like rgba... lets do that.
        let normal_sampled = (textureSample(texture[texture_meta.normal], texture_sampler[texture_meta.normal], input.uv_pos)).rgb;
        // Does this multiply with the normal from the vertices? Or overwrite? Probably multiply?
        // I think this needs some more work... This needs some reading...
        // https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/lighting-and-material/normal-mapping.html#sampling-normals
        //normal  = normalize( normal * ((normal_sampled * 2.0 - 1.0) * vec3f(normal_scale, normal_scale, 1.0)));
    }

    if DEBUG_OUTPUT_NORMALS {
        return normal_to_display_color(normal);
    }

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
