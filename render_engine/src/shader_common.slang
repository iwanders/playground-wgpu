
// Bindings for the camera uniform.
static const int CAMERA_UNIFORM_SET = 0;
static const int CAMERA_UNIFORM_BINDING = 0;

#ifdef REQUIRES_CAMERA_UNIFORM
struct CameraUniform {
    float4x4 view_proj;
    float3 camera_world_position;
};
[[vk::binding(CAMERA_UNIFORM_BINDING, CAMERA_UNIFORM_SET)]]
ConstantBuffer<CameraUniform> camera_uniform;
#endif


struct CommonVertexOutput {
    float4 clip_position :SV_Position;
    float3 color;
    float3 normal;
    float3 view_vector;
    float3 world_pos;
};
