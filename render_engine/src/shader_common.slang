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


struct CommonVertexOutput {
    float4 clip_position :SV_Position;
    float3 color;
    float3 normal;
    float3 view_vector;
    float3 world_pos;
};

static const float4x4 identity_float4x4 = float4x4(
    float4(1.0, 0.0, 0.0, 0.0),
    float4(0.0, 1.0, 0.0, 0.0),
    float4(0.0, 0.0, 1.0, 0.0),
    float4(0.0, 0.0, 0.0, 1.0)
);
