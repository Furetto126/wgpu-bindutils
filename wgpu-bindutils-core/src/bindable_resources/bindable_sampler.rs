use std::{cell::UnsafeCell, ops::{Index, IndexMut}};

use crate::bindable_resources::bindable::BindableField;

pub mod samp_kind {
    pub trait SamplerKind { const BINDING_TYPE: wgpu::SamplerBindingType; }
    pub struct Filtering; impl SamplerKind for Filtering { const BINDING_TYPE: wgpu::SamplerBindingType = wgpu::SamplerBindingType::Filtering; }
    pub struct NonFiltering; impl SamplerKind for NonFiltering { const BINDING_TYPE: wgpu::SamplerBindingType = wgpu::SamplerBindingType::NonFiltering; }
    pub struct Comparison; impl SamplerKind for Comparison { const BINDING_TYPE: wgpu::SamplerBindingType = wgpu::SamplerBindingType::Comparison; }
}

use samp_kind::*;

pub struct BindableSampler<Kind: SamplerKind = Filtering> {
    pub sampler: wgpu::Sampler,
    _kind: std::marker::PhantomData<Kind>
}

impl<Kind: SamplerKind> BindableSampler<Kind> {
    pub fn new(device: &wgpu::Device, desc: &wgpu::SamplerDescriptor) -> Self {
        let sampler = device.create_sampler(desc);
        Self { sampler, _kind: std::marker::PhantomData }
    }
}

impl<Kind: SamplerKind> BindableField for BindableSampler<Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(Kind::BINDING_TYPE),
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
        }
    }
}

pub struct BindableSamplerArray<const COUNT: usize, Kind: SamplerKind = Filtering> {
    samplers: Box<[wgpu::Sampler; COUNT]>, // SAFETY: This CANNOT be reassigned once initialized!
    sampler_refs: UnsafeCell<[&'static wgpu::Sampler; COUNT]>,
    _kind: std::marker::PhantomData<Kind>
}

impl<const COUNT: usize, Kind: SamplerKind> BindableSamplerArray<COUNT, Kind> {
    pub fn new(device: &wgpu::Device, descriptors: &[&wgpu::SamplerDescriptor]) -> Self {
        let mut samplers_vec = Vec::with_capacity(COUNT);
        for i in 0..COUNT {
            samplers_vec.push(device.create_sampler(descriptors[i]));
        }

        let samplers: Box<[wgpu::Sampler; COUNT]> = Box::new(samplers_vec.try_into().unwrap());
        let refs: [&wgpu::Sampler; COUNT] = samplers.each_ref();
        let refs_static: [&'static wgpu::Sampler; COUNT] = 
            unsafe { std::mem::transmute(refs) }; 

        Self {
            samplers: samplers,
            sampler_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }
}

impl<const COUNT: usize, Kind: SamplerKind> Index<usize> for BindableSamplerArray<COUNT, Kind> {
    type Output = wgpu::Sampler;

    fn index(&self, index: usize) -> &Self::Output {
        &self.samplers[index]
    }
}
impl<const COUNT: usize, Kind: SamplerKind> IndexMut<usize> for BindableSamplerArray<COUNT, Kind> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.samplers[index]
    }
}


impl<const COUNT: usize, Kind: SamplerKind> BindableField for BindableSamplerArray<COUNT, Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(Kind::BINDING_TYPE),
            count: std::num::NonZeroU32::new(COUNT as u32),
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        let refs: [&wgpu::Sampler; COUNT] = self.samplers.each_ref();

        unsafe {
            *self.sampler_refs.get() = std::mem::transmute(refs);

            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::SamplerArray(&*self.sampler_refs.get()),
            }   
        }
    }
}