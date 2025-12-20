// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> our_uniform: CameraUniform;


// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    //out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
    out.clip_position = our_uniform.view_proj * our_uniform.model_matrix*vec4<f32>(model.position, 1.0); // 2.
   out.normal = model.normal;

    return out;
}

//A: in.clip_position[1] == in.clip_position.y

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let color = in.normal * 0.5 + 0.5;
    // let color = in.color * in.normal.x;

    // let lightDirection = vec3f(0.5, -0.9, 0.1);
    // let shading = dot(lightDirection, in.normal);
    // let color = in.color * shading;
    //
    let lightColor1 = vec3f(1.0, 0.9, 0.6);
    let lightColor2 = vec3f(0.6, 0.9, 1.0);

    let lightDirection1 = vec3f(0.5, -0.9, 0.1);
    let lightDirection2 = vec3f(0.2, 0.4, 0.3);
    let shading1 = max(0.0, dot(lightDirection1, in.normal));
    let shading2 = max(0.0, dot(lightDirection2, in.normal));
    let shading = shading1 * lightColor1 + shading2 * lightColor2;
    let color = in.color * shading;
    return vec4<f32>(color, 1.0);
   // return vec4<f32>(in.color.r , in.color.g, in.color.b  , 1.0);
}
