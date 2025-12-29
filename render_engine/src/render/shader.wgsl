// This relies on camera & light uniform from common.

@binding(CAMERA_UNIFORM_BINDING) @group(CAMERA_UNIFORM_SET)
var<storage, read> camera_uniform : CameraUniformType;

@binding(LIGHT_UNIFORM_BINDING) @group(LIGHT_UNIFORM_SET)
var<storage, read> light_uniform : array<Light>;


//-- Next up is the actual fragment stuff.
const TEXTURE_UNIFORM_SET : u32 = 3;
const TEXTURE_UNIFORM_BINDING_TEXTURE: u32 = 0;
const TEXTURE_UNIFORM_BINDING_SAMPLER: u32 = 1;

@binding(TEXTURE_UNIFORM_BINDING_TEXTURE) @group(TEXTURE_UNIFORM_SET)
var texture : binding_array<texture_2d<f32>>;

@binding(TEXTURE_UNIFORM_BINDING_SAMPLER) @group(TEXTURE_UNIFORM_SET)
var texture_sampler : binding_array<sampler>;

@fragment
fn main( input : CommonVertexOutput) -> CommonFragmentOutput
{
    var output: CommonFragmentOutput;


    let N = normalize(input.normal);
   	let V = normalize(input.view_vector);

   	// Sample texture
   	// let baseColor = in.color;
    // https://github.com/Rust-GPU/VulkanShaderExamples/blob/b29a37eb46802b5ea6882af4808d6887fc184581/shaders/slang/texture/texture.slang#L58
    // let baseColor = texture_sampler[0].SampleLevel( in.uv_pos, 0).xyz;
    // let baseColor = texture_sampler.Sample( float3(in.uv_pos, 1.0)).rgb;
    // let baseColor = texture_sampler.Sample( in.uv_pos ).rgb;
    var baseColor : vec3<f32>;
   	// let texture_count : u32 = arrayLength(&texture_sampler);
    baseColor = (textureSample(texture[0], texture_sampler[0], input.uv_pos)).xyz;

    // let baseColor = float3(1.0, 0.0, 0.0);

   	let light_count : u32 = arrayLength(&light_uniform);


   	var color = vec3<f32>(0.0);
   	for (var i: u32 = 0; i < light_count; i++) {
   	    var this_light  = light_uniform[i];
   	    let light_type = this_light.light_type;
  		if (light_type == LIGHT_TYPE_OFF){
  		    continue;
  		}

  		// let light_color = this_light.color;
  		let light_color = vec3<f32>(1.0, 1.0, 1.0);
  		let L = Light_direction(&this_light, input.world_pos);
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
    output.color = vec4<f32>(corrected_color, 1.0);
    // output.color.x = 0;
   	return output;
    return output;
}
