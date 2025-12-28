

#include "./shader_common.slang"

RWStructuredBuffer<uint32_t> result;

#define genmain(NAME, A, B)  \
[shader("compute")] \
[numthreads(1,1,1)] \
void introspecter_##NAME(uint3 threadId : SV_DispatchThreadID)\
{\
    A our_instance;\
    result[0] = sizeof(A);\
    result[1] = sizeof(our_instance.B);\
    result[2] = uint64_t(uint64_t(&our_instance.B) - uint64_t(&our_instance));\
}
// #define CONCAT(A, B) A##B
// #define PPCAT(A, B) CONCAT(A, B) // Force expansion of arguments before concatenation


#define genmain_dispatch(C, A, B)\
    genmain(C, A, B)

#define genmain_entry(A, B)\
    genmain_dispatch(A##B, A, B)


genmain_entry(CameraUniform, view_proj)
genmain_entry(CameraUniform, camera_world_position)
