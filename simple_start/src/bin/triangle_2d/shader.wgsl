const positions: array<vec2<f32>, 3> = array(
    vec2(0.0, 0.5),
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
);

struct VsOut {
    @builtin(position)
    frag_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
    return VsOut(vec4(positions[index], 0.0, 1.0));
}

@fragment
fn fs_main(vs: VsOut) -> @location(0) vec4<f32> {
    // Seriously, how do we print here...
    // vs.frag_position.x -> pixel index from the left.
    // vs.frag_position.y -> pixel index from the top
    if (vs.frag_position.y > (128 + 32)) {
        return vec4(0.3, 0.2, 0.1, 1.0);
    } else {
        return vec4(1, abs(sin(f32((vs.frag_position.x -128 )) / 32)), cos(f32(vs.frag_position.y - 64) / 32), 1);
    }
}
