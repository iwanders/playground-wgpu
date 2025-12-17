// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(1) var gradientTexture: texture_2d<f32>;
@group(1) @binding(2) var textureSampler: sampler;

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
    //@location(2) texelCoords: vec2f,
    @location(2) uv: vec2f,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    //out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0); // A: Yes this is equivalent.
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0); // 2.
    out.normal = model.normal;
    // In plane.obj, the vertex xy coords range from -1 to 1
    // and we remap this to (0, 256), the size of our texture.
    // but our quad is from -0.5 to 0.5... ~and something is rotated :(~ ah my mesh was.
    out.uv = (model.position.xy + 0.5)  ;



    return out;
}

//A: in.clip_position[1] == in.clip_position.y

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let colorz = textureLoad(gradientTexture, vec2i(in.clip_position.xy), 0).rgb;
    //let colorz = textureLoad(gradientTexture, vec2i(in.uv), 0).rgb;
    let texelCoords = vec2i(in.uv * vec2f(textureDimensions(gradientTexture)));
    //let colorz = textureLoad(gradientTexture, texelCoords, 0).rgb;
    let colorz = textureSample(gradientTexture, textureSampler, in.uv).rgb;
    return vec4<f32>(colorz, 1.0);
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
