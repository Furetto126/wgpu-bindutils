/// The basic trait that types which want to participate in a [`BindableStruct`] must implement.<br>
/// This exposes a function <code>layout_entry</code> which statically returns a [`wgpu::BindGroupLayoutEntry`],<br>
/// and another function <code>bind_group_entry</code> which uses the instance to return a [`wgpu::BindGroupEntry`].
pub trait BindableField {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry;
    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_>;
}

/// The trait that will be implemented by the bind groups.
/// This exposes a function <code>bind_group_layout</code> which statically returns a [`wgpu::BindGroupLayout`] <br>
/// that describes the whole "shape" of the data with its bindings, using its internal [`BindableField`]s.<br>
/// It also exposes a function <code>bind_group</code> which should use its internal [`BindableField`]s to
/// construct a [`wgpu::BindGroup`] ready to be sent to the GPU.
pub trait BindableStruct {
    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
    fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}