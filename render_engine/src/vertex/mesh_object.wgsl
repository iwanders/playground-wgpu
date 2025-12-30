// This relies on ../shader_common.wgsl
// Use the camera uniform.
@binding(CAMERA_UNIFORM_BINDING) @group(CAMERA_UNIFORM_SET)
var<storage, read> camera_uniform : CameraUniformType;

//-- Next up is the actual mesh object.
const MESH_OBJECT_SET : u32 = 2;
const MESH_OBJECT_UNIFORM_BINDING: u32 = 0;
const MESH_OBJECT_INSTANCES_BINDING: u32 = 1;
const MESH_OBJECT_BINDING_NORMAL: u32 = 2;
const MESH_OBJECT_BINDING_COLOR: u32 = 3;
const MESH_OBJECT_BINDING_UV: u32 = 4;
const MESH_OBJECT_BINDING_TANGENT: u32 = 5;


struct MeshObjectMetaUniform {
    color_present: u32,
    normal_present: u32,
    uv_present: u32,
    tangent_present: u32,
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

@binding(MESH_OBJECT_BINDING_TANGENT) @group(MESH_OBJECT_SET) var<storage, read>
vertex_tangent : array<vec4<f32>>;


@vertex
fn main(in : VertexInput) ->  CommonVertexOutput {
    var out : CommonVertexOutput;
    // Because these are already arrays.
    let mesh_object_uniform = mesh_object_uniform[0];
    let camera_uniform = camera_uniform[0];

    // Short hands
    let view_proj = camera_uniform.view_proj;
    let camera_world_position = camera_uniform.camera_world_position;

    // Obtain the model location in the world.
    let model_matrix = mesh_object_instances[in.instanceID];
    // Transform the vertex from local frame to world frame.
    let world_position =  (model_matrix * vec4<f32>(in.position, 1.0));

    // Set the color to default ot white.
    out.color = vec3<f32>(1.0, 1.0, 1.0);
    if (mesh_object_uniform.color_present > 0) {
        out.color = vertex_color[in.vertexID].rgb;
    }

    // Retrieve the normal, and rotate it from local frame to world frame.
    if (mesh_object_uniform.normal_present > 0) {
        let normal = vertex_normal[in.vertexID];
        out.normal =  (model_matrix * vec4<f32>(normal, 1.0)).xyz;
    }
    // Retrieve the uv map.
    if (mesh_object_uniform.uv_present > 0) {
        out.uv_pos = vertex_uv[in.vertexID];
    }

    // Assign the clip position and other remainders to the output.
    out.clip_position = (view_proj * world_position);
    out.view_vector = camera_world_position - world_position.xyz;
    out.world_pos = world_position.xyz;
    return out;

}
