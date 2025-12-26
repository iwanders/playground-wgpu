// No true enums;  https://github.com/gpuweb/gpuweb/issues/4856
const LIGHT_TYPE_OFF = 0;
const LIGHT_TYPE_DIRECTIONAL = 1;
const LIGHT_TYPE_OMNI = 2;
const LIGHT_TYPE_AMBIENT = 3;

// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    camera_world_position: vec3f,
};
@group(0) @binding(0)
var<uniform> our_uniform: CameraUniform;


struct Light {
    position: vec3f,
    direction: vec3f,
    color: vec3f,
    intensity: f32,
    light_type: u32,
    // hardness_kd_ks: vec3f,
};
// No true methods: https://github.com/gpuweb/gpuweb/issues/4286
// And both 'this' and 'self' are reserved keywords
// And https://github.com/gfx-rs/wgpu/issues/5158  means we can't pass ptr<storage, Light, read>?
// And I can't figure out how to make an 'ptr<private, Light>' type... even just doing &light_uniform.lights[i] doesnt
// get me one.
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
}

struct LightUniform {
    lights: array<Light>, // arrayLength(&lights.point);
};
@group(1) @binding(1)
var<storage, read> light_uniform: LightUniform;



// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) viewDirection: vec3<f32>,
    @location(4) world_pos: vec3<f32>,
};


// https://github.com/eliemichel/LearnWebGPU-Code/blob/step105/resources/shader.wgsl
@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let world_position = our_uniform.model_matrix * vec4<f32>(in.position, 1.0);
   // out.color = model.color;
    out.color = vec3<f32>(1.0, 1.0, 1.0);
    //out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
    out.clip_position = our_uniform.view_proj * our_uniform.model_matrix * vec4<f32>(in.position, 1.0); // 2.
    out.normal = (our_uniform.model_matrix * vec4f(in.normal, 0.0)).xyz;
    out.viewDirection = our_uniform.camera_world_position - world_position.xyz;
    out.world_pos = (our_uniform.model_matrix * vec4<f32>(in.position, 1.0)).xyz;
    return out;
}


// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let color = in.normal * 0.5 + 0.5;
    // let color = in.color * in.normal.x;

    // let lightDirection = vec3f(0.5, -0.9, 0.1);
    // let shading = dot(lightDirection, in.normal);
    // let color = in.color * shading;
    //
    // Compute shading
	let N = normalize(in.normal);
	let V = normalize(in.viewDirection);

	// Sample texture
	let baseColor = vec3(1.0, 1.0, 1.0);

	let light_count = arrayLength(&light_uniform.lights);
	var color = vec3f(0.0);
	for (var i: u32 = 0; i < light_count; i++) {
	    let this_light  = light_uniform.lights[i];
	    let light_type = this_light.light_type;
		if (light_type == LIGHT_TYPE_OFF){
		    continue;
		}

		let lightColor = this_light.color;
		let L = Light_direction(this_light, in.world_pos);
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
	return vec4f(corrected_color, 1.0);
}
