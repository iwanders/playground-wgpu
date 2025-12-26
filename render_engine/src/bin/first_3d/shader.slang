/*

https://shader-slang.org/slang/user-guide/spirv-target-specific#spir-v-specific-attributes
[[vk::binding(binding: int, set: int = 0)]]
Similar to binding layout qualifier in Vulkan. It specifies the uniform buffer binding point, and the descriptor set for Vulkan.

==>> BINDING INDEX FIRST, THEN SET
*/


enum LightType
{
    LIGHT_TYPE_OFF = 0,
    LIGHT_TYPE_DIRECTIONAL = 1,
    LIGHT_TYPE_OMNI = 2,
    LIGHT_TYPE_AMBIENT = 3,
};

// Vertex shader
struct CameraUniform {
    float4x4 view_proj;
    float4x4 model_matrix;
    float3 camera_world_position;
};
[[vk::binding(0, 0)]]
StructuredBuffer<CameraUniform> our_uniform;

struct Light {
    float3 position;
    float3 direction;
    float3 color;
    float intensity;
    LightType light_type;
    // hardness_kd_ks: vec3f,

    float3 light_direction(float3 at_point)
    {
        switch (light_type) {
            case LightType::LIGHT_TYPE_DIRECTIONAL: {
                return direction;
            }
            case LightType::LIGHT_TYPE_OMNI: {
                let difference = position - at_point;
                return normalize(difference);
            }
            default: {
                return float3(0.0, 0.0, 0.0);
            }
        }
    }
};
struct LightUniform {
    Light lights[];
};
[[vk::binding(0, 1)]]
StructuredBuffer<Light> light_uniform;


struct VertexInput {
    float3 position;
    float3 normal;
    float3 color;
};

struct VertexOutput {
    //@builtin(position) clip_position: vec4<f32>,
    float4          clip_position     : SV_Position;
    float3 color;
    float3 normal;
    float3 viewDirection;
    float3 world_pos;
};



[shader("vertex")]
VertexOutput vertexMain(VertexInput in)
{
    VertexOutput out;
    let our_uniform= our_uniform[0];
    let world_position = mul(our_uniform.model_matrix , float4(in.position, 1.0));
   // out.color = model.color;
    out.color = float3(1.0, 1.0, 1.0);
    //out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
    out.clip_position = mul(our_uniform.view_proj , mul(our_uniform.model_matrix , float4(in.position, 1.0))); // 2.
    out.normal = mul(our_uniform.model_matrix, float4(in.normal, 0.0)).xyz;
    out.viewDirection = our_uniform.camera_world_position - world_position.xyz;
    out.world_pos = mul(our_uniform.model_matrix , float4(in.position, 1.0)).xyz;
    return out;
}

// Output of the fragment shader
struct Fragment
{
    float4 color;
};
[shader("fragment")]
Fragment fragmentMain( VertexOutput in) : SV_Target
{
    let N = normalize(in.normal);
   	let V = normalize(in.viewDirection);

   	// Sample texture
   	let baseColor = float3(1.0, 1.0, 1.0);

   	let light_count = light_uniform.getCount();
   	var color = float3(0.0);
   	for (var i: uint = 0; i < light_count; i++) {
   	    let this_light  = light_uniform[i];
   	    let light_type = this_light.light_type;
  		if (light_type == LightType::LIGHT_TYPE_OFF){
  		    continue;
  		}

  		let lightColor = this_light.color;
  		let L = this_light.light_direction(in.world_pos);
  		let hardness = this_light.intensity ;
  		let kd = 0.5;
  		let ks = 0.3;
  		// let kd = light_uniform.lights[i].hardness_kd_ks.y; // diffuse effect
  		// let ks = light_uniform.lights[i].hardness_kd_ks.z; // specular effect
  		let R = reflect(-L, N); // equivalent to 2.0 * dot(N, L) * N - L

  		let diffuse = max(0.0, dot(L, N)) * lightColor;

  		// We clamp the dot product to 0 when it is negative
  		let RoV = max(0.0, dot(R, V));
  		let specular = pow(RoV, hardness);

  		color += baseColor * (kd * diffuse + ks * specular);
   	}
   	// color /= f32(light_count - 2);

   	// Gamma-correction
   	let corrected_color= color;
   	// let corrected_color = pow(color, vec3f(2.2));
    Fragment output;
    output.color = float4(corrected_color, 1.0);
    // output.color.x = 0;
   	return output;
}
