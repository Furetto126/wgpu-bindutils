use std::{cell::UnsafeCell, ops::Index};

use crate::bindable_resources::{bindable::BindableField, tex_opts::kind::TextureKind};

pub mod tex_opts {
    pub mod dim {
        pub trait Dimension { 
            const VIEW_DIMENSION: wgpu::TextureViewDimension;
            fn physical_dimension() -> wgpu::TextureDimension {
                match Self::VIEW_DIMENSION {
                    wgpu::TextureViewDimension::D1 => wgpu::TextureDimension::D1,
                    wgpu::TextureViewDimension::D3 => wgpu::TextureDimension::D3,
                    _ => wgpu::TextureDimension::D2
                }
            }
        }
        pub struct D1; impl Dimension for D1 { const VIEW_DIMENSION: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D1; }
        pub struct D2; impl Dimension for D2 { const VIEW_DIMENSION: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2; }
        pub struct D2Array; impl Dimension for D2Array { const VIEW_DIMENSION: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2Array; }
        pub struct D3; impl Dimension for D3 { const VIEW_DIMENSION: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D3; }
        pub struct Cube; impl Dimension for Cube { const VIEW_DIMENSION: wgpu::TextureViewDimension = wgpu::TextureViewDimension::Cube; }
    }

    pub mod fmt {
        pub trait TexFormat { 
            const FORMAT: wgpu::TextureFormat;
            const SAMPLE_TYPE: wgpu::TextureSampleType;
        }
        pub struct Rgba8Unorm<const FILTERABLE: bool>;
        impl<const FILTERABLE: bool> TexFormat for Rgba8Unorm<FILTERABLE> {
            const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
            const SAMPLE_TYPE: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: FILTERABLE };
        }
        pub struct Rgba32Float;
        impl TexFormat for Rgba32Float {
            const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
            const SAMPLE_TYPE: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
        }
        pub struct R32Uint;
        impl TexFormat for R32Uint {
            const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;
            const SAMPLE_TYPE: wgpu::TextureSampleType = wgpu::TextureSampleType::Uint;
        }
    }

    pub mod access {
        pub trait StorageAccess { const ACCESS: wgpu::StorageTextureAccess; }
        pub struct ReadOnly; impl StorageAccess for ReadOnly { const ACCESS: wgpu::StorageTextureAccess = wgpu::StorageTextureAccess::ReadOnly; }
        pub struct WriteOnly; impl StorageAccess for WriteOnly { const ACCESS: wgpu::StorageTextureAccess = wgpu::StorageTextureAccess::WriteOnly; }
        pub struct ReadWrite; impl StorageAccess for ReadWrite { const ACCESS: wgpu::StorageTextureAccess = wgpu::StorageTextureAccess::ReadWrite; }
    }

    pub mod kind {
        use super::fmt::TexFormat;
        use super::dim::{Dimension, D2};
        use super::access::{StorageAccess, ReadOnly};

        pub trait TextureKind {
            fn binding_type() -> wgpu::BindingType;
            fn usage() -> wgpu::TextureUsages;
        }

        pub struct TexSampled<F: TexFormat, D: Dimension = D2, const MULTISAMPLED: bool = false>(
            std::marker::PhantomData<(F, D)>
        );
        impl<F: TexFormat, D: Dimension, const MS: bool> TextureKind for TexSampled<F, D, MS> {
            fn binding_type() -> wgpu::BindingType {
                wgpu::BindingType::Texture {
                    sample_type: F::SAMPLE_TYPE,
                    view_dimension: D::VIEW_DIMENSION,
                    multisampled: MS,
                }
            }

            fn usage() -> wgpu::TextureUsages {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
            }
        }

        pub struct TexStorage<F: TexFormat, A: StorageAccess = ReadOnly, D: Dimension = D2>(
            std::marker::PhantomData<(F, A, D)>
        );
        impl<F: TexFormat, A: StorageAccess, D: Dimension> TextureKind for TexStorage<F, A, D> {
            fn binding_type() -> wgpu::BindingType {
                wgpu::BindingType::StorageTexture {
                    access: A::ACCESS,
                    format: F::FORMAT,
                    view_dimension: D::VIEW_DIMENSION,
                }
            }

            fn usage() -> wgpu::TextureUsages {
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST
            }
        }
    }
}

use tex_opts::*;

pub struct BindableTexture<Kind: kind::TextureKind> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    _kind: std::marker::PhantomData<Kind>
}

impl<F: fmt::TexFormat, D: dim::Dimension, const MS: bool> BindableTexture<kind::TexSampled<F, D, MS>> {
    pub fn new_sampled(device: &wgpu::Device, size: wgpu::Extent3d, label: Option<&str>) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: if MS { 4 } else { 1 },
            dimension: D::physical_dimension(),
            format: F::FORMAT,
            usage: kind::TexSampled::<F, D, MS>::usage(),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(D::VIEW_DIMENSION),
            ..Default::default()
        });

        Self {
            texture,
            view,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<F: fmt::TexFormat, A: access::StorageAccess, D: dim::Dimension> BindableTexture<kind::TexStorage<F, A, D>> {
    pub fn new_storage(device: &wgpu::Device, size: wgpu::Extent3d, label: Option<&str>) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: D::physical_dimension(),
            format: F::FORMAT,
            usage: kind::TexStorage::<F, A, D>::usage(),
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(D::VIEW_DIMENSION),
            ..Default::default()
        });

        Self {
            texture,
            view,
            _kind: std::marker::PhantomData,
        }
    }
}

impl<Kind: kind::TextureKind> BindableField for BindableTexture<Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: Kind::binding_type(),
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&self.view),
        }
    }
}

pub struct BindableTextureArray<const COUNT: usize, Kind: kind::TextureKind> {
    textures: [wgpu::Texture; COUNT],
    views: Box<[wgpu::TextureView; COUNT]>, // SAFETY: This CANNOT be reassigned once initialized!
    view_refs: UnsafeCell<[&'static wgpu::TextureView; COUNT]>,
    _kind: std::marker::PhantomData<Kind>
}

impl<const COUNT: usize, F: fmt::TexFormat, D: dim::Dimension, const MS: bool>
    BindableTextureArray<COUNT, kind::TexSampled<F, D, MS>> {
    pub fn new_sampled(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        label: Option<&str>
    ) -> Self {
        let mut textures  = Vec::with_capacity(COUNT);
        let mut views_vec = Vec::with_capacity(COUNT);
        for _ in 0..COUNT {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: if MS { 4 } else { 1 },
                dimension: D::physical_dimension(),
                format: F::FORMAT,
                usage: kind::TexSampled::<F, D, MS>::usage(),
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(D::VIEW_DIMENSION),
                ..Default::default()
            });

            textures.push(texture);
            views_vec.push(view);
        }

        let views: Box<[wgpu::TextureView; COUNT]> = Box::new(views_vec.try_into().unwrap());
        let refs: [&wgpu::TextureView; COUNT] = views.each_ref();
        let refs_static: [&'static wgpu::TextureView; COUNT] =
            unsafe { std::mem::transmute(refs) };

        Self {
            textures: textures.try_into().unwrap(),
            views,
            view_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }

    pub fn set_texture(&mut self, i: usize, typed_texture: BindableTexture<kind::TexSampled<F, D, MS>>) {
        self.textures[i] = typed_texture.texture;
        self.views[i] = typed_texture.view;
    }
}

impl<const COUNT: usize, F: fmt::TexFormat, A: access::StorageAccess, D: dim::Dimension>
    BindableTextureArray<COUNT, kind::TexStorage<F, A, D>> {
    pub fn new_storage(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        label: Option<&str>
    ) -> Self {
        let mut textures  = Vec::with_capacity(COUNT);
        let mut views_vec = Vec::with_capacity(COUNT);
        for _ in 0..COUNT {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: D::physical_dimension(),
                format: F::FORMAT,
                usage: kind::TexStorage::<F, A, D>::usage(),
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(D::VIEW_DIMENSION),
                ..Default::default()
            });

            textures.push(texture);
            views_vec.push(view);
        }

        let views: Box<[wgpu::TextureView; COUNT]> = Box::new(views_vec.try_into().unwrap());
        let refs: [&wgpu::TextureView; COUNT] = views.each_ref();
        let refs_static: [&'static wgpu::TextureView; COUNT] =
            unsafe { std::mem::transmute(refs) };

        Self {
            textures: textures.try_into().unwrap(),
            views,
            view_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }


    pub fn set_texture(&mut self, i: usize, typed_texture: BindableTexture<kind::TexStorage<F, A, D>>) {
        self.textures[i] = typed_texture.texture;
        self.views[i] = typed_texture.view;
    }
}

impl<const COUNT: usize, Kind: kind::TextureKind> Index<usize> for BindableTextureArray<COUNT, Kind> {
    type Output = wgpu::Texture;

    fn index(&self, index: usize) -> &Self::Output {
        &self.textures[index]
    }
}

impl<const COUNT: usize, Kind: kind::TextureKind> BindableField for BindableTextureArray<COUNT, Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: Kind::binding_type(),
            count: std::num::NonZeroU32::new(COUNT as u32),
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        let refs: [&wgpu::TextureView; COUNT] = self.views.each_ref();

        unsafe {
            *self.view_refs.get() = std::mem::transmute(refs);

            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureViewArray(&*self.view_refs.get()),
            }   
        }
    }
}