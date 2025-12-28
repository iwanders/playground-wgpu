// https://shader-slang.org/docs/coming-from-hlsl/#buffer-layouts
// but -fvk-use-c-layout violates the 16-byte padding requirements in all the structs...


struct StructField{
    uint size_;
    uint offset_;
    uint8_t name_length_;
    uint8_t name_[32];
    __init(uint size, uint offset, uint8_t use_name[] ) {
        size_ = size;
        offset_ = offset;
        name_length_ = uint8_t(use_name.getCount());
        for (uint i = 0; i < use_name.getCount(); i++ ){
            name_[i] = use_name[i];
        }
    }
};


// Bindings for the camera uniform.
static const int CAMERA_UNIFORM_SET = 0;
static const int CAMERA_UNIFORM_BINDING = 0;

struct CameraUniform{
    float4x4 view_proj;
    float3 camera_world_position;


    static int introspectStructSize() {
        return sizeof(CameraUniform);
    }
    static StructField[2] introspectField()
    {
        CameraUniform z;
        return {
            StructField(sizeof(z.view_proj), uint((uint64_t(&z.view_proj) - uint64_t(&z))), {'v', 'i', 'e', 'w', '_', 'p', 'r', 'o', 'j'}),
            StructField(sizeof(z.camera_world_position), uint(uint64_t(&z.camera_world_position) - uint64_t(&z)), {'a'})
        };
    }
}

#ifdef REQUIRES_CAMERA_UNIFORM
[[vk::binding(CAMERA_UNIFORM_BINDING, CAMERA_UNIFORM_SET)]]
StructuredBuffer<CameraUniform> camera_uniform;
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
