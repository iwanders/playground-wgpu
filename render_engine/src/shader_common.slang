// https://shader-slang.org/docs/coming-from-hlsl/#buffer-layouts
// but -fvk-use-c-layout violates the 16-byte padding requirements in all the structs...

// Bindings for the camera uniform.
static const int CAMERA_UNIFORM_SET = 0;
static const int CAMERA_UNIFORM_BINDING = 0;

struct ViewUniform{
    float4x4 view_proj;
    float3 camera_world_position;
    int _pad;
}


#ifdef REQUIRES_CAMERA_UNIFORM
[[vk::binding(CAMERA_UNIFORM_BINDING, CAMERA_UNIFORM_SET)]]
StructuredBuffer<ViewUniform> camera_uniform;
#endif




enum LightType
{
    LIGHT_TYPE_OFF = 0,
    LIGHT_TYPE_DIRECTIONAL = 1,
    LIGHT_TYPE_OMNI = 2,
    LIGHT_TYPE_AMBIENT = 3,
};


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

static const int LIGHT_UNIFORM_SET = 1;
static const int LIGHT_UNIFORM_BINDING = 0;
#ifdef REQUIRES_LIGHT_UNIFORM
[[vk::binding(LIGHT_UNIFORM_BINDING, LIGHT_UNIFORM_SET)]]
StructuredBuffer<Light> light_uniform;
#endif

struct CommonVertexOutput {
    float4 clip_position :SV_Position;
    float3 color;
    float3 normal;
    float3 view_vector;
    float3 world_pos;
};

struct CommonFragmentOutput
{
    float4 color;
};

static const float4x4 identity_float4x4 = float4x4(
    float4(1.0, 0.0, 0.0, 0.0),
    float4(0.0, 1.0, 0.0, 0.0),
    float4(0.0, 0.0, 1.0, 0.0),
    float4(0.0, 0.0, 0.0, 1.0)
);
