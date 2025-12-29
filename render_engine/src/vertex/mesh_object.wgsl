// #define REQUIRES_CAMERA_UNIFORM 1
// #include "../shader_common.slang"


// Bindings for the camera uniform.
const CAMERA_UNIFORM_SET : u32 = 0;
const CAMERA_UNIFORM_BINDING : u32 = 0;

struct ViewUniform{
    view_proj: mat4x4<f32>,
    camera_world_position: vec3<f32>,
    pad: u32,
}

@binding(CAMERA_UNIFORM_BINDING) @group(CAMERA_UNIFORM_SET)
var<storage, read> camera_uniform : array<ViewUniform>;

// From common, vertex output;
struct CommonVertexOutput
{
    @builtin(position) clip_position  : vec4<f32>,
    @location(0) color : vec3<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) view_vector : vec3<f32>,
    @location(3) world_pos : vec3<f32>,
    @location(4) uv_pos : vec2<f32>,
};



struct VertexInput
{
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instanceID: u32,
    @builtin(vertex_index) vertexID: u32,
};

//-- Next up is the actual mesh object.
const MESH_OBJECT_SET : u32 = 2;
const MESH_OBJECT_UNIFORM_BINDING: u32 = 0;
const MESH_OBJECT_INSTANCES_BINDING: u32 = 1;
const MESH_OBJECT_BINDING_NORMAL: u32 = 2;
const MESH_OBJECT_BINDING_COLOR: u32 = 3;
const MESH_OBJECT_BINDING_UV: u32 = 4;


struct MeshObjectMetaUniform {
    color_present: u32,
    normal_present: u32,
    uv_present: u32,
};

@binding(MESH_OBJECT_UNIFORM_BINDING) @group(MESH_OBJECT_SET)
var<storage, read> mesh_object_uniform  : array<MeshObjectMetaUniform>;


@binding(MESH_OBJECT_INSTANCES_BINDING) @group(MESH_OBJECT_SET) var<storage, read>
mesh_object_instances : array<mat4x4<f32>>;

@binding(MESH_OBJECT_BINDING_NORMAL) @group(MESH_OBJECT_SET) var<storage, read>
vertex_normal : array<vec3<f32>>;

@binding(MESH_OBJECT_BINDING_COLOR) @group(MESH_OBJECT_SET) var<storage, read>
vertex_color : array<vec4<f32>>;

@binding(MESH_OBJECT_BINDING_UV) @group(MESH_OBJECT_SET) var<storage, read>
vertex_uv : array<vec2<f32>>;


@vertex
fn main(in : VertexInput) ->  CommonVertexOutput {
    var out : CommonVertexOutput;

    let model_matrix = mesh_object_instances[in.instanceID];
    let world_position =  (model_matrix * vec4<f32>(in.position, 1.0));
    let mesh_object_uniform= mesh_object_uniform[0];
    let camera_uniform = camera_uniform[0];
    out.color = vec3<f32>(1.0, 1.0, 1.0);
    if (mesh_object_uniform.color_present > 0) {
        out.color = vertex_color[in.vertexID].xyz;
    }
    var normal = vec3<f32>(0.0, 0.0, 0.0);
    if (mesh_object_uniform.normal_present > 0) {
        normal = vertex_normal[in.vertexID];
    }
    if (mesh_object_uniform.uv_present > 0) {
        out.uv_pos = vertex_uv[in.vertexID];
    }
    // normal = vertex_normals[in.vertex_index];
    out.clip_position = (camera_uniform .view_proj * (model_matrix * vec4<f32>(in.position, 1.0)));
    out.normal =  (model_matrix * vec4<f32>(normal, 0.0)).xyz;
    out.view_vector = camera_uniform .camera_world_position - world_position.xyz;
    out.world_pos =  (model_matrix * vec4<f32>(in.position, 1.0)).xyz;
    return out;

}
