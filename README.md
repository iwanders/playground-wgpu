# playground_wgpu

Some explorations around how things get rendered on the screen. I started with making a triangle with wgpu, then
explored various other API's in the [exploration](./exploration/) directory before focussing on [render_engine](./render_engine/)
which is a simple framework for rendering (simple gltfs) with wgpu.

This readme is mostly a bunch of notes and pointers for myself.

## On physically based rendering.
For PBR rendering, [s2013_pbs_physics_math_notes.pdf](https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf), gives a really good overview of how this works, and the
[gltf 2.0 specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#appendix-b-brdf-implementation) is 
also worth going through.

Most of my notes on this are in [permalinked(/ old) shader.wgsl](https://github.com/iwanders/playground-wgpu/blob/11d4332f9c82575788e7902fd08dba92fd01186d/render_engine/src/fragment/shader.wgsl#L3).

The [DamagedHelmet](https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/DamagedHelmet) gltf sample asset
rendered with `cargo r --release --bin damaged_helmet_pbr` at [ab32ccb54e](https://github.com/iwanders/playground-wgpu/commit/ab32ccb54e84a5a259345b8189cbf0f0cc3a4c4f) looks like this:

![damaged helmet render](https://github.com/user-attachments/assets/43f2cdb0-3346-4d75-bc21-ef78612d716b)

Note on loading textures; some of them are in linear space, some of them are in srgb space. It is important to get this 
right when loading them such that sampling from the texture works correctly, the gltf specification always states which
space it is in.

Using the normal map in the pbr shader was surprisingly involved, requiring calculation of tangents in mikktspace before 
the normal map can be used in the fragment shader. The [gltf spec](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#implementation:~:text=When%20tangents%20are%20not%20specified%2C%20client%20implementations%20SHOULD%20calculate%20tangents%20using%20default%20MikkTSpace%20algorithms%20with%20the%20specified%20vertex%20positions%2C%20normals%2C%20and%20texture%20coordinates%20associated%20with%20the%20normal%20texture.) does mention this. This [tutorial](https://eliemichel.github.io/LearnWebGPU/basic-3d-rendering/lighting-and-material/normal-mapping.html) is helpful for understanding the overall idea.

## On coordinate frames.
GLTF does NOT follow the same coordinate system as Blender. Blender has z up, while gltf has y up. There's a good image on this,
permalink to it in in [godot's documentation](https://github.com/godotengine/godot-docs/blob/fd4deee99f7361145546f069bdd80f64af3ff401/tutorials/3d/introduction_to_3d.rst#coordinate-system). This describes the various frames used by different programs. Unreal actually
[switched](https://dev.epicgames.com/documentation/en-us/fortnite/leftupforward-coordinate-system-in-unreal-editor-for-fortnite)
to a Left-Up-Forward coordinate system, which is also what GLTF uses, and what is used in my render_engine crate.


---
License is MIT.
