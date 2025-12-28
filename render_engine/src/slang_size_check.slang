

#include "./shader_common.slang"

RWStructuredBuffer<uint32_t> result;

#define generate_introspecter_impl(NAME, A, B)  \
[shader("compute")] \
[numthreads(1,1,1)] \
void introspecter_##NAME(uint3 threadId : SV_DispatchThreadID)\
{\
    A our_instance;\
    result[0] = sizeof(A);\
    result[1] = sizeof(our_instance.B);\
    result[2] = uint64_t(uint64_t(&our_instance.B) - uint64_t(&our_instance));\
}



#define generate_introspecter(A, B)\
    generate_introspecter_impl(A##B, A, B)

// This was the ONLY way I could figure out how to 'exfiltrate' the 'view_proj' segment in a way that ends up in the
// spv file AND is locked into using the correct field name and location.
generate_introspecter(ViewUniform, view_proj)
generate_introspecter(ViewUniform, camera_world_position)
generate_introspecter(ViewUniform, _pad)
