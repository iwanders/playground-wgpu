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
    out.clip_position = our_uniform.view_proj * our_uniform.model_matrix*vec4<f32>(in.position, 1.0); // 2.
    out.normal = (our_uniform.model_matrix * vec4f(in.normal, 0.0)).xyz;
    out.viewDirection = our_uniform.camera_world_position - world_position.xyz;
    return out;
}

//A: in.clip_position[1] == in.clip_position.y

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
		let lightColor = light_uniform.lights[i].color;
		let L = normalize(light_uniform.lights[i].direction);
		let hardness = light_uniform.lights[i].intensity ;
		let kd = 0.5;
		let ks = 0.9;
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
	let corrected_color = pow(color, vec3f(2.2));
	return vec4f(corrected_color, 1.0);
}
