// They reserved 'this' and 'self'... so lets just use 'me'.


// Bindings for the camera uniform.
const CAMERA_UNIFORM_SET : u32 = 0;
const CAMERA_UNIFORM_BINDING : u32 = 0;
alias CameraUniformType = array<ViewUniform>;
struct ViewUniform {
    view_proj: mat4x4<f32>,
    camera_world_position: vec3<f32>,
    pad: u32,
}
// @binding(CAMERA_UNIFORM_BINDING) @group(CAMERA_UNIFORM_SET)
// var<storage, read> camera_uniform : CameraUniformType;



// From common, vertex output;
struct CommonVertexOutput {
    @builtin(position) clip_position  : vec4<f32>,
    @location(0) color : vec3<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) view_vector : vec3<f32>,
    @location(3) world_pos : vec3<f32>,
    @location(4) uv_pos : vec2<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instanceID: u32,
    @builtin(vertex_index) vertexID: u32,
};


struct CommonFragmentOutput{
    @location(0) color: vec4<f32>,
};


// -- todo
// No enums; https://github.com/gpuweb/gpuweb/issues/4856
alias LightType = u32;
const LIGHT_TYPE_OFF : LightType = 0;
const LIGHT_TYPE_DIRECTIONAL : LightType = 1;
const LIGHT_TYPE_OMNI : LightType = 2;
const LIGHT_TYPE_AMBIENT : LightType = 3;


struct Light {
     @location(0) position:  vec3<f32> ,
     @location(1) direction: vec3<f32>,
     @location(2) color: vec3<f32>,
     @location(3) intensity: f32 ,
     @location(4) light_type: LightType,
     // hardness_kd_ks: vec3f,
};
fn Light_direction(me: ptr<function,Light>,  at_point: vec3<f32>) -> vec3<f32> {
    switch((*me).light_type)
    {
        case LIGHT_TYPE_DIRECTIONAL:
            {
                return normalize((*me).direction);
            }
        case LIGHT_TYPE_OMNI:
            {
                return normalize((*me).position - at_point);
            }
        default :
            {
                return vec3<f32>(0.0f, 0.0f, 0.0f);
            }
    }
}

const LIGHT_UNIFORM_SET : u32 = 1;
const LIGHT_UNIFORM_BINDING : u32 = 0;
// @binding(LIGHT_UNIFORM_BINDING) @group(LIGHT_UNIFORM_SET)
// var<storage, read> light_uniform : array<Light>;


// -- Texture
//
//
const TEXTURE_UNIFORM_SET : u32 = 3;
const TEXTURE_UNIFORM_BINDING_TEXTURE: u32 = 0;
const TEXTURE_UNIFORM_BINDING_SAMPLER: u32 = 1;
const TEXTURE_UNIFORM_META: u32 = 2;

alias TextureType = u32;
const TEXTURE_TYPE_NONE : TextureType = 0;
const TEXTURE_TYPE_BASE_COLOR : TextureType = 1;
const TEXTURE_TYPE_METALLIC_ROUGHNESS : TextureType = 2;
const TEXTURE_TYPE_OCCLUSION : TextureType = 3;
const TEXTURE_TYPE_NORMAL : TextureType = 4;
const TEXTURE_TYPE_EMISSIVE : TextureType = 5;

struct TextureUniform {
     @location(0) base_color: u32,
     @location(1) metallic_roughness: u32,
     @location(2) occlusion: u32,
     @location(3) normal: u32,
     @location(4) emissive: u32,
};
// @binding(TEXTURE_UNIFORM_META) @group(TEXTURE_UNIFORM_SET)
// var<storage, read> texture_uniform : array<TextureUniform>;
