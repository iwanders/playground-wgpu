#[derive(Debug, Clone)]
pub struct BindGroupLayoutDescriptorOwned {
    pub label: Option<String>,
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutDescriptorOwned {
    pub fn into_descriptor(&self) -> wgpu::BindGroupLayoutDescriptor<'_> {
        wgpu::BindGroupLayoutDescriptor {
            label: self.label.as_ref().map(|v| v.as_str()),
            entries: &self.entries,
        }
    }
}
