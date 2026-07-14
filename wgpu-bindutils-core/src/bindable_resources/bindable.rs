pub trait BindableStruct {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
    fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}

pub trait BindableField {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry;
    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_>;
}