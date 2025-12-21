use naga::ShaderStage;

pub fn compile_shader(glsl: &str, stage: ShaderStage) -> Result<Vec<u8>, anyhow::Error> {
    // Parse the source into a Module.
    use naga::front::glsl::{Frontend, Options};
    let mut frontend = Frontend::default();
    let options = Options::from(stage);
    let module: naga::Module = frontend.parse(&options, glsl)?;

    // Validate the module.
    // Validation can be made less restrictive by changing the ValidationFlags.
    let module_info: naga::valid::ModuleInfo = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .subgroup_stages(naga::valid::ShaderStages::all())
    .subgroup_operations(naga::valid::SubgroupOperationSet::all())
    .validate(&module)?;

    // Translate the module.
    use naga::back::spv;
    let options = spv::Options::default();
    let outu32 = spv::write_vec(&module, &module_info, &options, None)?;

    use zerocopy::IntoBytes;
    Ok(outu32.as_bytes().to_vec())
}
