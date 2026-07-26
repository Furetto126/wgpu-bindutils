use std::{cell::UnsafeCell, ops::{Index, IndexMut}};

use crate::bindable_resources::bindable::BindableField;

pub mod samp_kind {
    pub trait SamplerKind {
        const BINDING_TYPE: wgpu::SamplerBindingType;
        fn matches_descriptor(desc: &wgpu::SamplerDescriptor) -> bool;
        fn default_descriptor<'a>() -> wgpu::SamplerDescriptor<'a>;
    }
    pub struct Filtering; 
    impl SamplerKind for Filtering {
        const BINDING_TYPE: wgpu::SamplerBindingType = wgpu::SamplerBindingType::Filtering;
        fn matches_descriptor(desc: &wgpu::SamplerDescriptor) -> bool {
            desc.compare.is_none() 
        }
        fn default_descriptor<'a>() -> wgpu::SamplerDescriptor<'a> {
            wgpu::SamplerDescriptor {
                label: Some("Default Filtering Sampler Descriptor."),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                compare: None,
                ..Default::default()
            }
        }
    }
    pub struct NonFiltering;
    impl SamplerKind for NonFiltering {
        const BINDING_TYPE: wgpu::SamplerBindingType = wgpu::SamplerBindingType::NonFiltering;
        fn matches_descriptor(desc: &wgpu::SamplerDescriptor) -> bool {
            desc.mag_filter == wgpu::FilterMode::Nearest
                && desc.min_filter == wgpu::FilterMode::Nearest
                && desc.mipmap_filter == wgpu::MipmapFilterMode::Nearest
                && desc.compare.is_none()
        }
        fn default_descriptor<'a>() -> wgpu::SamplerDescriptor<'a> {
            wgpu::SamplerDescriptor {
                label: Some("Default Filtering Sampler Descriptor."),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                compare: None,
                ..Default::default()
            }
        }
    }
    pub struct Comparison; 
    impl SamplerKind for Comparison {
        const BINDING_TYPE: wgpu::SamplerBindingType = wgpu::SamplerBindingType::Comparison;
        fn matches_descriptor(desc: &wgpu::SamplerDescriptor) -> bool {
            desc.compare.is_some()
        }
        fn default_descriptor<'a>() -> wgpu::SamplerDescriptor<'a> {
            wgpu::SamplerDescriptor {
                label: Some("Default Filtering Sampler Descriptor."),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::Always),
                ..Default::default()
            }
        }
    }
}

use samp_kind::*;

/// The implementation of [BindableField] for a [wgpu::Sampler] binding resource.
pub struct BindableSampler<Kind: SamplerKind = Filtering> {
    sampler: wgpu::Sampler,
    _kind: std::marker::PhantomData<Kind>
}

impl<Kind: SamplerKind> BindableSampler<Kind> {
    /// Create a [`BindableSampler`] with a specified [`wgpu::SamplerDescriptor`].
    pub fn new(device: &wgpu::Device, desc: &wgpu::SamplerDescriptor) -> Self {
        // Check validity.
        assert!(Kind::matches_descriptor(desc), "Provided SamplerDescriptor does not match with expected Kind.");

        let sampler = device.create_sampler(desc);
        Self { sampler, _kind: std::marker::PhantomData }
    }


    /// Creates a [`BindableSampler`] from a [`wgpu::Sampler`]
    /// 
    /// # Correctness concerns
    /// This function is marked *unsafe* because currently there is no way
    /// to check at runtime information<br> about a[`wgpu::Sampler`] object.<br>
    /// Thus it would be impossible to [`assert`] correctness and type-safety while also providing
    /// user-friendly error messages.<br>
    /// However, based on how this resulting [`BindableSampler`] is used, [`wgpu`] may throw validation errors
    /// as soon as the mismatch is noticed.
    pub unsafe fn from_sampler(sampler: &wgpu::Sampler) -> Self {
        Self { sampler: sampler.clone(), _kind: std::marker::PhantomData }
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

/// The implementation of [BindableField] for an array of [wgpu::Sampler] binding resources.
pub struct BindableSamplerArray<const MAX_SAMPLERS: usize, Kind: SamplerKind = Filtering> {
    samplers: Box<[wgpu::Sampler; MAX_SAMPLERS]>, // SAFETY: This CANNOT be reassigned once initialized!
    sampler_refs: UnsafeCell<[&'static wgpu::Sampler; MAX_SAMPLERS]>,
    _kind: std::marker::PhantomData<Kind>
}

impl<const MAX_SAMPLERS: usize, Kind: SamplerKind> BindableSamplerArray<MAX_SAMPLERS, Kind> {
    /// Creates a [`BindableSamplerArray`] where each [`wgpu::SamplerDescriptor`] is used for its 
    /// corresponding [`wgpu::Sampler`] 
    pub fn new(device: &wgpu::Device, descriptors: &[&wgpu::SamplerDescriptor]) -> Self {
        let mut samplers_vec = Vec::with_capacity(MAX_SAMPLERS);
        for i in 0..MAX_SAMPLERS {
            assert!(Kind::matches_descriptor(descriptors[i]), "Provided SamplerDescriptor at index {i} does not match with expected Kind.");
            samplers_vec.push(device.create_sampler(descriptors[i]));
        }

        let samplers: Box<[wgpu::Sampler; MAX_SAMPLERS]> = Box::new(samplers_vec.try_into().unwrap());
        let refs: [&wgpu::Sampler; MAX_SAMPLERS] = samplers.each_ref();
        let refs_static: [&'static wgpu::Sampler; MAX_SAMPLERS] = 
            unsafe { std::mem::transmute(refs) };

        Self {
            samplers: samplers,
            sampler_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }

    /// Creates a [`BindableSamplerArray`] from an existing [`wgpu::Sampler`] vector.
    /// 
    /// # Correctness concerns
    /// This function is marked *unsafe* because currently there is no way
    /// to check at runtime information<br> about a [`wgpu::Sampler`] object.<br>
    /// Thus it would be impossible to [`assert`] correctness and type-safety while also providing
    /// user-friendly error messages.<br>
    /// However, based on how the resulting [`BindableSampler`]s are used, [`wgpu`] may throw validation errors
    /// as soon as the mismatch is noticed.
    /// 
    /// Note that this will *panic* if the supplied [`wgpu::Sampler`] vector is longer than MAX_SAMPLERS.
    pub unsafe fn from_samplers(device: &wgpu::Device, samplers: &Vec<wgpu::Sampler>) -> Self {
        assert!(samplers.len() < MAX_SAMPLERS,
            "Failed to create BindableSamplerArray: provided sampler array length ({}) was greater than MAX_SAMPLERS ({MAX_SAMPLERS})", samplers.len());
        
        let mut samplers_vec: Vec<wgpu::Sampler> = vec![];
        for s in samplers {
            samplers_vec.push(s.clone());
        }

        for _ in samplers.len()..MAX_SAMPLERS {
            samplers_vec.push(device.create_sampler(&Kind::default_descriptor()).clone());
        }

        let samplers_box: Box<[wgpu::Sampler; MAX_SAMPLERS]> = Box::new(samplers_vec.try_into().unwrap());
        let refs: [&wgpu::Sampler; MAX_SAMPLERS] = samplers_box.each_ref();
        let refs_static: [&'static wgpu::Sampler; MAX_SAMPLERS] = 
            unsafe { std::mem::transmute(refs) };

        Self {
            samplers: samplers_box,
            sampler_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }
}

impl<const MAX_SAMPLERS: usize, Kind: SamplerKind> Index<usize> for BindableSamplerArray<MAX_SAMPLERS, Kind> {
    type Output = wgpu::Sampler;

    fn index(&self, index: usize) -> &Self::Output {
        &self.samplers[index]
    }
}

// Is this needed or useful? Would it allow for anything to break?
impl<const MAX_SAMPLERS: usize, Kind: SamplerKind> IndexMut<usize> for BindableSamplerArray<MAX_SAMPLERS, Kind> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.samplers[index]
    }
}

impl<const MAX_SAMPLERS: usize, Kind: SamplerKind> BindableField for BindableSamplerArray<MAX_SAMPLERS, Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(Kind::BINDING_TYPE),
            count: std::num::NonZeroU32::new(MAX_SAMPLERS as u32),
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        let refs: [&wgpu::Sampler; MAX_SAMPLERS] = self.samplers.each_ref();

        unsafe {
            *self.sampler_refs.get() = std::mem::transmute(refs);

            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::SamplerArray(&*self.sampler_refs.get()),
            }   
        }
    }
}