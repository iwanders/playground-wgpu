# playground_wgpu

Some explorations around how things get rendered on the screen.

I initially started with `simple_start_wgpu`, which just rendered to a image file, then built
`interactivity_wgpu` that has a movable camera. Then I switched to exploring some other api's, 
and explored [ash](https://github.com/ash-rs/ash) and [vulkano](https://github.com/vulkano-rs/vulkano)
in the `simple_start_{ash,vulkano}` directories respectively, this gave a better understanding of
how things work under the hood. This was very insightful but also quite involved. I also switched
to writing [slang](https://shader-slang.org/) shaders.


After those initial epxlorations I built `render_engine`, which is a bit more of an actual framework
where various parts come together to draw an object. Here I hit a snag with using slang as this used
combined texture samplers in the spir-v output, which is not something wgpu produced. As a stop-gap
solution I used `slangc` to compile to wgsl and then imported that, which split the combination into
a separate texture and sampler again, but this was obviously less than ideal, so I switched it all
back to wgsl, even though I liked having struct methods and proper enums.

License is MIT.
