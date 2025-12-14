// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;


// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    // out.clip_position = vec4<f32>(model.position, 1.0); // Q: What's this sorcery? Does this automatically expand?
    out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    return out;
}

//A: in.clip_position[1] == in.clip_position.y

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.clip_position[1] > 160) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    if (in.clip_position.x > 160 && i32(in.clip_position.x) % 2 == 0 && i32(in.clip_position.y) % 2 == 0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(in.color, 0.9);
}
// What's that @location(0) at the end here?
// https://www.w3.org/TR/WGSL/#input-output-locations
//  Each input-output location can store a value up to 16 bytes in size.
//  Each user-defined input and output must have an explicitly specified IO location

// Q: Where does the interpolation actually happen? Like the fragment shader doesn't do that, neither does the vertex
// shader?
//
// A: The rasterization is the very heart ❤️ of the 3D rendering algorithm implemented by a GPU.
// It transforms a primitive (a point, a line or a triangle) into a series of fragments, that correspond to the pixels
// covered by the primitive. It interpolates any extra attribute output by the vertex shader, such that each fragment
// receives a value for all attributes.
//
// From: https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/hello-triangle.html#primitive-pipeline-state
