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

    // Tangent, Bitangent and Normal (TBN) vectors for handling the normal map defined in mikktspace.
    @location(5) tangent_w : vec3<f32>,
    @location(6) bitangent_w : vec3<f32>,
    @location(7) normal_w : vec3<f32>,
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

/// Light direction is from at_point towards the light.
fn Light_direction(me: ptr<function,Light>,  at_point: vec3<f32>) -> vec3<f32> {
    switch((*me).light_type)
    {
        case LIGHT_TYPE_DIRECTIONAL:
            {
                return normalize(-(*me).direction);
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
/// Determines the light intensity at a certain point for this light, accounting for falloff.
fn Light_intensity(me: ptr<function,Light>,  at_point: vec3<f32>) -> f32 {
    switch((*me).light_type)
    {
        case LIGHT_TYPE_OMNI:
            {
                // This falls off...
                let distance = length((*me).position - at_point);
                // This is... well not ideal, it becomes very very large when the light is close, and also doesn't
                // take into account anything like a light range, or other falloff properties, but it is good enough
                // for the simple point lights I have right now.
                let remaining = 1.0 / (distance * distance + 0.01);
                return (*me).intensity * remaining;
            }
        default :
            {
                return (*me).intensity;
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

//-----------------------------------------------------
// Constants
const PI_F: f32 = 3.141592653589793;



//-----------------------------------------------------
// Color space utils.
// https://github.com/KhronosGroup/glTF-Sample-Renderer/blob/e6b052db89fb2adbaf31da4565a08265c96c2b9f/source/Renderer/shaders/tonemapping.glsl#L26
const GAMMA: f32 = 2.2;
const INV_GAMMA: f32 = 1.0 / GAMMA;
fn linear_to_srgb(color: vec3f) -> vec3f {
    return pow(color, vec3(INV_GAMMA));
}
fn srgb_to_linear(color: vec3f) -> vec3f {
    return  pow(color, vec3f(GAMMA));
}
fn tonemap_khronos_pbr_neutral(color_in: vec3f) -> vec3f{
    const startCompression: f32 = 0.8 - 0.04;
    const desaturation: f32 = 0.15;
    var color = color_in;

    let x = min(color.r, min(color.g, color.b));

    let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
    color -= offset;

    let peak = max(color.r, max(color.g, color.b));
    if (peak < startCompression) { return color;
    }

    let d = 1.0 - startCompression;
    let newPeak = 1. - d * d / (peak + d - startCompression);
    color *= newPeak / peak;

    let g = 1. - 1. / (desaturation * (peak - newPeak) + 1.);
    return mix(color, newPeak * vec3f(1.0, 1.0, 1.0), g);

}

//-----------------------------------------------------
// Check https://www.w3.org/TR/WGSL/#numeric-builtin-functions first for built in functions.
// Utility functions
fn heaviside(a: f32) -> f32 {
    // Heaviside: 1.0 if x > 0, 0.0 if x <= 0
    // Built in step(edge, x); Returns 1.0 if edge ≤ x, and 0.0 otherwise. Component-wise when T is a vector.
    // bah.
    if (a > 0) {
        return 1.0;
    } else {
        return 0.0;
    }
}


//-----------------------------------------------------
// Third party functions
//
//
// This inverts a 4x4 matrix numerically. It's less than ideal, we are almost always inverting homogeneous matrices that
// could be inverted by transposing R and doing -R * v, but the scaling (mostly non-uniform scaling) makes this harder.
// It should still be possible to do something that's more elegant though, by identifying the individual scale components
// with some decomposition, then doing the homogenous inverse properly, and then re-applying the scale? This is a rabbit
// hole to go into now, and I just want to get my normal maps working.
// From https://github.com/gfx-rs/wgpu/blob/aba9161b72c028aa8a1ce15aabd92e3c3cdb2da3/naga/src/back/wgsl/polyfill/inverse/inverse_4x4_f32.wgsl
// Which is MIT OR Apache-2 license.
fn _naga_inverse_4x4_f32(m: mat4x4<f32>) -> mat4x4<f32> {
   let sub_factor00: f32 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
   let sub_factor01: f32 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
   let sub_factor02: f32 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
   let sub_factor03: f32 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
   let sub_factor04: f32 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
   let sub_factor05: f32 = m[2][0] * m[3][1] - m[3][0] * m[2][1];
   let sub_factor06: f32 = m[1][2] * m[3][3] - m[3][2] * m[1][3];
   let sub_factor07: f32 = m[1][1] * m[3][3] - m[3][1] * m[1][3];
   let sub_factor08: f32 = m[1][1] * m[3][2] - m[3][1] * m[1][2];
   let sub_factor09: f32 = m[1][0] * m[3][3] - m[3][0] * m[1][3];
   let sub_factor10: f32 = m[1][0] * m[3][2] - m[3][0] * m[1][2];
   let sub_factor11: f32 = m[1][1] * m[3][3] - m[3][1] * m[1][3];
   let sub_factor12: f32 = m[1][0] * m[3][1] - m[3][0] * m[1][1];
   let sub_factor13: f32 = m[1][2] * m[2][3] - m[2][2] * m[1][3];
   let sub_factor14: f32 = m[1][1] * m[2][3] - m[2][1] * m[1][3];
   let sub_factor15: f32 = m[1][1] * m[2][2] - m[2][1] * m[1][2];
   let sub_factor16: f32 = m[1][0] * m[2][3] - m[2][0] * m[1][3];
   let sub_factor17: f32 = m[1][0] * m[2][2] - m[2][0] * m[1][2];
   let sub_factor18: f32 = m[1][0] * m[2][1] - m[2][0] * m[1][1];

   var adj: mat4x4<f32>;
   adj[0][0] =   (m[1][1] * sub_factor00 - m[1][2] * sub_factor01 + m[1][3] * sub_factor02);
   adj[1][0] = - (m[1][0] * sub_factor00 - m[1][2] * sub_factor03 + m[1][3] * sub_factor04);
   adj[2][0] =   (m[1][0] * sub_factor01 - m[1][1] * sub_factor03 + m[1][3] * sub_factor05);
   adj[3][0] = - (m[1][0] * sub_factor02 - m[1][1] * sub_factor04 + m[1][2] * sub_factor05);
   adj[0][1] = - (m[0][1] * sub_factor00 - m[0][2] * sub_factor01 + m[0][3] * sub_factor02);
   adj[1][1] =   (m[0][0] * sub_factor00 - m[0][2] * sub_factor03 + m[0][3] * sub_factor04);
   adj[2][1] = - (m[0][0] * sub_factor01 - m[0][1] * sub_factor03 + m[0][3] * sub_factor05);
   adj[3][1] =   (m[0][0] * sub_factor02 - m[0][1] * sub_factor04 + m[0][2] * sub_factor05);
   adj[0][2] =   (m[0][1] * sub_factor06 - m[0][2] * sub_factor07 + m[0][3] * sub_factor08);
   adj[1][2] = - (m[0][0] * sub_factor06 - m[0][2] * sub_factor09 + m[0][3] * sub_factor10);
   adj[2][2] =   (m[0][0] * sub_factor11 - m[0][1] * sub_factor09 + m[0][3] * sub_factor12);
   adj[3][2] = - (m[0][0] * sub_factor08 - m[0][1] * sub_factor10 + m[0][2] * sub_factor12);
   adj[0][3] = - (m[0][1] * sub_factor13 - m[0][2] * sub_factor14 + m[0][3] * sub_factor15);
   adj[1][3] =   (m[0][0] * sub_factor13 - m[0][2] * sub_factor16 + m[0][3] * sub_factor17);
   adj[2][3] = - (m[0][0] * sub_factor14 - m[0][1] * sub_factor16 + m[0][3] * sub_factor18);
   adj[3][3] =   (m[0][0] * sub_factor15 - m[0][1] * sub_factor17 + m[0][2] * sub_factor18);

   let det = (m[0][0] * adj[0][0] + m[0][1] * adj[1][0] + m[0][2] * adj[2][0] + m[0][3] * adj[3][0]);

   return adj * (1 / det);
}
