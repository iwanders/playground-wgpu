#language slang 2026


struct CameraUniform
{
     float4x4 view_proj;
     float4x4 model_matrix;
     float3 camera_world_position;
};

[[vk::binding(0, 0)]]
uniform  CameraUniform our_uniform;




struct Light
{
     float3 color;
     float3 direction;
     float3 hardness_kd_ks;
};
struct LightUniform {
     StructuredBuffer<Light> lights ;
};


[[vk::binding(0, 1)]]
uniform LightUniform light_uniform;

struct VertexIn
{
    [[vk::location(0)]] float3 position;
    [[vk::location(1)]] float4 normal;
    [[vk::location(2)]] float4 color;
};


struct VertexOut
{
	float4 position : SV_POSITION;
	[[vk::location(0)]] float4 color;
	[[vk::location(1)]] float3 normal;
    [[vk::location(2)]] float3 viewDirection;
};

[shader("vertex")]
VertexOut vertexMain(VertexIn input)
{
	VertexOut output;
	// output.color =  input.color ;
	// output.position = mul(our_uniform.view_proj,  float4(input.position, 1.0));

    let world_position = mul(our_uniform.model_matrix, float4(input.position, 1.0));
   // out.color = model.color;
    output.color = float4(1.0, 1.0, 1.0, 1.0);
    //out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
    output.position = mul(mul(our_uniform.view_proj , our_uniform.model_matrix), float4(input.position, 1.0)); // 2.
    output.normal = mul(our_uniform.model_matrix , input.normal ).xyz;
    output.viewDirection = our_uniform.camera_world_position - world_position.xyz;


	return output;
}


[shader("fragment")]
float4 fragmentMain(VertexOut input)
{
    // return input.color ;
    let N = normalize(input.normal);
	let V = normalize(input.viewDirection);

	// Sample texture
	let baseColor = float3(1.0, 1.0, 1.0);

	let light_count =  light_uniform.lights.getCount();
	var color = float3(0.0);
	for (var i: int = 0; i < light_count; i++) {
		let lightColor = light_uniform.lights[i].color;
		let L = normalize(light_uniform.lights[i].direction);
		let hardness = light_uniform.lights[i].hardness_kd_ks.x;
		let kd = light_uniform.lights[i].hardness_kd_ks.y; // diffuse effect
		let ks = light_uniform.lights[i].hardness_kd_ks.z; // specular effect
		let R = reflect(-L, N); // equivalent to 2.0 * dot(N, L) * N - L

		let diffuse = max(0.0, dot(L, N)) * lightColor;

		// We clamp the dot product to 0 when it is negative
		let RoV = max(0.0, dot(R, V));
		let specular = pow(RoV, hardness);

		color += baseColor * kd * diffuse + ks * specular;
	}
	// color /= f32(light_count - 2);

	// Gamma-correction
	let corrected_color = pow(color, float3(2.2));
	return float4(corrected_color, 1.0);
}
