/*

https://shader-slang.org/slang/user-guide/spirv-target-specific#spir-v-specific-attributes
[[vk::binding(binding: int, set: int = 0)]]
Similar to binding layout qualifier in Vulkan. It specifies the uniform buffer binding point, and the descriptor set for Vulkan.

==>> BINDING INDEX FIRST, THEN SET
*/

// Snag...
// How do we NOT provide a combined texture sampler?
// https://shader-slang.org/slang/user-guide/spirv-target-specific#combined-texture-sampler
// oh... https://github.com/shader-slang/slang/issues/6471
// hmm and https://github.com/gfx-rs/wgpu/issues/4342
//
// Okay, so this is a bit of a rabbit hole...
//  Options:
//      - Hack https://docs.rs/wgpu/latest/wgpu/enum.BindingResource.html to have a CombinedSampler
//      - Go from slang -> wgsl, which splits the sampler (and implicity consumes binding+1)

#define REQUIRES_CAMERA_UNIFORM 1
#define REQUIRES_LIGHT_UNIFORM 1
#include "../shader_common.slang"


static const int TEXTURE_UNIFORM_SET = 3;
static const int TEXTURE_UNIFORM_BINDING_TEXTURE = 0;
static const int TEXTURE_UNIFORM_BINDING_SAMPLER = 1;
// var texture_array_top: binding_array<texture_2d<f32>>;

// [[vk::binding(TEXTURE_UNIFORM_BINDING_TEXTURE, TEXTURE_UNIFORM_SET)]]
// Texture2DArray texture_array;

[[vk::binding(TEXTURE_UNIFORM_BINDING_TEXTURE, TEXTURE_UNIFORM_SET)]]
Sampler2D texture_sampler ;

// Output of the fragment shader
[shader("fragment")]
CommonFragmentOutput fragmentMain( CommonVertexOutput in) : SV_Target
{
    CommonFragmentOutput output;
    let N = normalize(in.normal);
   	let V = normalize(in.view_vector);

   	// Sample texture
   	// let baseColor = in.color;
    // https://github.com/Rust-GPU/VulkanShaderExamples/blob/b29a37eb46802b5ea6882af4808d6887fc184581/shaders/slang/texture/texture.slang#L58
    // let baseColor = texture_sampler[0].SampleLevel( in.uv_pos, 0).xyz;
    // let baseColor = texture_sampler.Sample( float3(in.uv_pos, 1.0)).rgb;
    let baseColor = texture_sampler.Sample( in.uv_pos ).rgb;

    // let baseColor = float3(1.0, 0.0, 0.0);

   	let light_count = light_uniform.getCount();

    if (false) {
        output.color.r = in.uv_pos.x;
        output.color.g = in.uv_pos.y;
        // output.color.b = 0.0;
        // output.color.a = 1.0;
        return output;
    }

   	var color = float3(0.0);
   	for (var i: uint = 0; i < light_count; i++) {
   	    let this_light  = light_uniform[i];
   	    let light_type = this_light.light_type;
  		if (light_type == LightType::LIGHT_TYPE_OFF){
  		    continue;
  		}

  		// let light_color = this_light.color;
  		let light_color = float3(1.0, 1.0, 1.0);
  		let L = this_light.light_direction(in.world_pos);
  		let hardness = this_light.intensity ;
  		let kd = 0.5;
  		let ks = 0.3;
  		// let kd = light_uniform.lights[i].hardness_kd_ks.y; // diffuse effect
  		// let ks = light_uniform.lights[i].hardness_kd_ks.z; // specular effect
  		let R = reflect(-L, N); // equivalent to 2.0 * dot(N, L) * N - L

  		let diffuse = max(0.0, dot(L, N)) * light_color;

  		// We clamp the dot product to 0 when it is negative
  		let RoV = max(0.0, dot(R, V));
  		let specular = pow(RoV, hardness);

  		color += baseColor * (kd * diffuse + ks * specular);
   	}
   	// color /= f32(light_count - 2);

   	// Gamma-correction
   	let corrected_color= color;
   	// let corrected_color = pow(color, vec3f(2.2));
    output.color = float4(corrected_color, 1.0);
    // output.color.x = 0;
   	return output;
}
