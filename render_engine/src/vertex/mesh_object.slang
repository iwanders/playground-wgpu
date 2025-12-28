#define REQUIRES_CAMERA_UNIFORM 1
#include "../shader_common.slang"


// Must match mesh_object.rs
static const int MESH_OBJECT_SET = 2;
static const int MESH_OBJECT_UNIFORM_BINDING = 0;
static const int MESH_OBJECT_INSTANCES_BINDING = 1;
static const int MESH_OBJECT_BINDING_NORMAL = 2;
static const int MESH_OBJECT_BINDING_COLOR = 3;
static const int MESH_OBJECT_BINDING_UV = 4;


struct MeshObjectMetaUniform {
    uint color_present;
    uint normal_present;
    uint uv_present;
};
[[vk::binding(MESH_OBJECT_UNIFORM_BINDING, MESH_OBJECT_SET)]]
StructuredBuffer<MeshObjectMetaUniform> mesh_object_uniform;


[[vk::binding(MESH_OBJECT_INSTANCES_BINDING, MESH_OBJECT_SET)]]
StructuredBuffer<float4x4> mesh_object_instances;


struct VertexInput {
    float3 position;
    uint vertexID : SV_VulkanVertexID;
    uint instanceID : SV_VulkanInstanceID;
};

[[vk::binding(MESH_OBJECT_BINDING_NORMAL, MESH_OBJECT_SET)]]
StructuredBuffer<float3> vertex_normal;

[[vk::binding(MESH_OBJECT_BINDING_COLOR, MESH_OBJECT_SET)]]
StructuredBuffer<float4> vertex_color;

[[vk::binding(MESH_OBJECT_BINDING_UV, MESH_OBJECT_SET)]]
StructuredBuffer<float2> vertex_uv;

[shader("vertex")]
CommonVertexOutput vertexMain(VertexInput in)
{
    CommonVertexOutput out;

    let model_matrix = mesh_object_instances[in.instanceID];
    let world_position = mul(model_matrix , float4(in.position, 1.0));
    let mesh_object_uniform= mesh_object_uniform[0];
    let camera_uniform = camera_uniform[0];
    out.color = float3(1.0, 1.0, 1.0);
    if (mesh_object_uniform.color_present > 0) {
        out.color = vertex_color[in.vertexID].xyz;
    }
    var normal = float3(0.0, 0.0, 0.0);
    if (mesh_object_uniform.normal_present > 0) {
        normal = vertex_normal[in.vertexID];
    }
    if (mesh_object_uniform.uv_present > 0) {
        out.uv_pos = vertex_uv[in.vertexID];
    }
    // normal = vertex_normals[in.vertex_index];
    out.clip_position = mul(camera_uniform .view_proj , mul(model_matrix , float4(in.position, 1.0)));
    out.normal = mul(model_matrix, float4(normal, 0.0)).xyz;
    out.view_vector = camera_uniform .camera_world_position - world_position.xyz;
    out.world_pos = mul(model_matrix , float4(in.position, 1.0)).xyz;
    return out;
}
