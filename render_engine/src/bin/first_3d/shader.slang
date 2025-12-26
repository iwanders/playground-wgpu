/*

https://shader-slang.org/slang/user-guide/spirv-target-specific#spir-v-specific-attributes
[[vk::binding(binding: int, set: int = 0)]]
Similar to binding layout qualifier in Vulkan. It specifies the uniform buffer binding point, and the descriptor set for Vulkan.

==>> BINDING INDEX FIRST, THEN SET
*/

// No true enums;  https://github.com/gpuweb/gpuweb/issues/4856
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
// @group(0) @binding(0)
// var<uniform> our_uniform: CameraUniform;
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
// No true methods: https://github.com/gpuweb/gpuweb/issues/4286
// And both 'this' and 'self' are reserved keywords
// And https://github.com/gfx-rs/wgpu/issues/5158  means we can't pass ptr<storage, Light, read>?
// And I can't figure out how to make an 'ptr<private, Light>' type... even just doing &light_uniform.lights[i] doesnt
// get me one.
/*
fn Light_direction(light:  Light , at_point: vec3f) -> vec3f {
    switch light.light_type {
        case LIGHT_TYPE_DIRECTIONAL: {
            return light.direction;
        }
        case LIGHT_TYPE_OMNI: {
            let difference = light.position - at_point;
            return normalize(difference);
        }
        case LIGHT_TYPE_AMBIENT, LIGHT_TYPE_OFF: {
            return vec3f(0.0, 0.0, 0.0);
        }
        default: {
            return vec3f(0.0, 0.0, 0.0);
        }
    }
    }*/


struct LightUniform {
    Light lights[]; // arrayLength(&lights.point);
};
// @group(1) @binding(1)
// var<storage, read> light_uniform: LightUniform;
[[vk::binding(0, 1)]]
StructuredBuffer<Light> light_uniform;


// Vertex shader
// struct VertexInput {
//     @location(0) position: float3,
//     @location(1) normal: float3,
//     @location(2) color: float3,
// };
struct VertexInput {
    float3 position;
    float3 normal;
    float3 color;
};

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: float3,
//     @location(1) normal: float3,
//     @location(2) viewDirection: float3,
//     @location(4) world_pos: float3,
// };
struct VertexOutput {
    //@builtin(position) clip_position: vec4<f32>,
    float4          clip_position     : SV_Position;
    float3 color;
    float3 normal;
    float3 viewDirection;
    float3 world_pos;
};


// https://github.com/eliemichel/LearnWebGPU-Code/blob/step105/resources/shader.wgsl
// @vertex
// fn vs_main(
//     in: VertexInput,
// ) -> VertexOutput {
//     var out: VertexOutput;
//     let world_position = our_uniform.model_matrix * vec4<f32>(in.position, 1.0);
//    // out.color = model.color;
//     out.color = float3(1.0, 1.0, 1.0);
//     //out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
//     out.clip_position = our_uniform.view_proj * our_uniform.model_matrix * vec4<f32>(in.position, 1.0); // 2.
//     out.normal = (our_uniform.model_matrix * vec4f(in.normal, 0.0)).xyz;
//     out.viewDirection = our_uniform.camera_world_position - world_position.xyz;
//     out.world_pos = (our_uniform.model_matrix * vec4<f32>(in.position, 1.0)).xyz;
//     return out;
// }

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


// // Fragment shader
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     // let color = in.normal * 0.5 + 0.5;
//     // let color = in.color * in.normal.x;

//     // let lightDirection = vec3f(0.5, -0.9, 0.1);
//     // let shading = dot(lightDirection, in.normal);
//     // let color = in.color * shading;
//     //
//     // Compute shading
// 	let N = normalize(in.normal);
// 	let V = normalize(in.viewDirection);

// 	// Sample texture
// 	let baseColor = vec3(1.0, 1.0, 1.0);

// 	let light_count = arrayLength(&light_uniform.lights);
// 	var color = vec3f(0.0);
// 	for (var i: u32 = 0; i < light_count; i++) {
// 	    let this_light  = light_uniform.lights[i];
// 	    let light_type = this_light.light_type;
// 		if (light_type == LIGHT_TYPE_OFF){
// 		    continue;
// 		}

// 		let lightColor = this_light.color;
// 		let L = Light_direction(this_light, in.world_pos);
// 		let hardness = this_light.intensity ;
// 		let kd = 0.5;
// 		let ks = 0.3;
// 		// let kd = light_uniform.lights[i].hardness_kd_ks.y; // diffuse effect
// 		// let ks = light_uniform.lights[i].hardness_kd_ks.z; // specular effect
// 		let R = reflect(-L, N); // equivalent to 2.0 * dot(N, L) * N - L

// 		let diffuse = max(0.0, dot(L, N)) * lightColor;

// 		// We clamp the dot product to 0 when it is negative
// 		let RoV = max(0.0, dot(R, V));
// 		let specular = pow(RoV, hardness);

// 		color += baseColor * kd * diffuse + ks * specular;
// 	}
// 	// color /= f32(light_count - 2);

// 	// Gamma-correction
// 	let corrected_color= color;
// 	// let corrected_color = pow(color, vec3f(2.2));
// 	return vec4f(corrected_color, 1.0);
// }


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

  		color += baseColor * kd * diffuse + ks * specular;
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
