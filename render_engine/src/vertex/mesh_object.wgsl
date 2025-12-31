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

    // This is gross, I thought we could avoid the matrix inverse on the model matrix, given that it is a homogenous matrix
    // that at worst has a scaled rotation matrix, but the scaling may be uniform, which makes it a lot harder to do an easy
    // transpose & -Rv trick to invert it, so we're using the polyfill from wgsl here;
    // https://github.com/gfx-rs/wgpu/blob/aba9161b72c028aa8a1ce15aabd92e3c3cdb2da3/naga/src/back/wgsl/polyfill/inverse/inverse_4x4_f32.wgsl
    // Ideally we'd either constrain scales to be uniform and do a bit of math to do the analytical inverse.
    let normal_matrix = transpose(_naga_inverse_4x4_f32(model_matrix));

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


    if (mesh_object_uniform.tangent_present > 0) {
        let tangent = normalize(vertex_tangent[in.vertexID]);
        let normal = normalize(vertex_normal[in.vertexID]);

        // This follows https://github.com/KhronosGroup/glTF-Sample-Renderer/blob/e6b052db89fb2adbaf31da4565a08265c96c2b9f/source/Renderer/shaders/primitive.vert#L135-L148
        out.tangent_w = (model_matrix * vec4f(tangent.xyz, 0.0)).xyz;
        out.normal_w = normalize((normal_matrix * vec4f(normal, 0.0)).xyz);
        out.bitangent_w = cross(out.normal_w, out.tangent_w) * tangent.w;
    }


    // Assign the clip position and other remainders to the output.
    out.clip_position = (view_proj * world_position);
    out.view_vector = camera_world_position - world_position.xyz;
    out.world_pos = world_position.xyz;
    return out;

}
