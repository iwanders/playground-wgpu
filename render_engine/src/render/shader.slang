/*

https://shader-slang.org/slang/user-guide/spirv-target-specific#spir-v-specific-attributes
[[vk::binding(binding: int, set: int = 0)]]
Similar to binding layout qualifier in Vulkan. It specifies the uniform buffer binding point, and the descriptor set for Vulkan.

==>> BINDING INDEX FIRST, THEN SET

Set/Group: A collection of related bindings.



https://github.com/shader-slang/slang/tree/master/examples/gpu-printing#gpu-printing



    StructuredBuffer stride is 16?
*/
#define REQUIRES_CAMERA_UNIFORM 1
#define REQUIRES_LIGHT_UNIFORM 1
#include "../shader_common.slang"

// Output of the fragment shader
[shader("fragment")]
CommonFragmentOutput fragmentMain( CommonVertexOutput in) : SV_Target
{
    let N = normalize(in.normal);
   	let V = normalize(in.view_vector);

   	// Sample texture
   	let baseColor = in.color;

   	let light_count = light_uniform.getCount();
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
    CommonFragmentOutput output;
    output.color = float4(corrected_color, 1.0);
    // output.color.x = 0;
   	return output;
}
