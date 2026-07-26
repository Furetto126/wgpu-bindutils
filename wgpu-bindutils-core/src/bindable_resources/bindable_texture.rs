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
            const BYTES_PER_PIXEL: u32;
        }
        pub struct Rgba8Unorm<const FILTERABLE: bool>;
        impl<const FILTERABLE: bool> TexFormat for Rgba8Unorm<FILTERABLE> {
            const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
            const SAMPLE_TYPE: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: FILTERABLE };
            const BYTES_PER_PIXEL: u32 = 4;
        }
        pub struct Rgba32Float;
        impl TexFormat for Rgba32Float {
            const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
            const SAMPLE_TYPE: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
            const BYTES_PER_PIXEL: u32 = 16;
        }
        pub struct R32Uint;
        impl TexFormat for R32Uint {
            const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;
            const SAMPLE_TYPE: wgpu::TextureSampleType = wgpu::TextureSampleType::Uint;
            const BYTES_PER_PIXEL: u32 = 4;
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

/// The implementation of [BindableField] for a [wgpu::Texture] binding resource.
pub struct BindableTexture<Kind: kind::TextureKind> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    _kind: std::marker::PhantomData<Kind>
}

impl<F: fmt::TexFormat, D: dim::Dimension, const MS: bool> BindableTexture<kind::TexSampled<F, D, MS>> {
    /// Creates a [`BindableTexture`] with a specified size.
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
            format: Some(F::FORMAT),
            dimension: Some(D::VIEW_DIMENSION),
            ..Default::default()
        });

        Self { texture, view, _kind: std::marker::PhantomData, }
    }

    /// Creates a [`BindableTexture`] from an existing [`wgpu::Texture`].
    /// 
    /// Note that this will **panic** if the supplied texture does not match the [`BindableTexture`]'s type signature.
    pub fn from_sampled(texture: &wgpu::Texture) -> Self {
        Self::check_validity(texture);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(F::FORMAT),
            dimension: Some(D::VIEW_DIMENSION),
            ..Default::default()
        });

        Self { texture: texture.clone(), view, _kind: std::marker::PhantomData, }
    }

    /// Updates the underlying [`wgpu::Texture`] using a &\[u8\] as pixel data.
    /// 
    /// Note that this will **panic** if the supplied pixel data does not match the number of bytes
    /// the [`BindableTexture`] expects.
    pub fn update(&self, queue: &wgpu::Queue, size: wgpu::Extent3d, data: &[u8]) {
        let expected_bytes = F::BYTES_PER_PIXEL * size.width * size.height * size.depth_or_array_layers;
        assert_eq!(
            data.len() as u32, expected_bytes,
            "Data length does not match expected size for this BindableTexture\n\nExpected: {}\nReceived: {}", data.len(), expected_bytes  
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.width * F::BYTES_PER_PIXEL),
                rows_per_image: Some(size.height)
            },
            size
        );
    }
    
    // Assert validity
    fn check_validity(texture: &wgpu::Texture) {
        assert_eq!(texture.format(), F::FORMAT, "Texture format ({:?}) does not match TexSampled<F, ..>", texture.format());
        assert_eq!(texture.dimension(), D::physical_dimension(), "Texture dimension ({:?}) does not match TexSampled<.., D, ..>", texture.dimension());
        assert_eq!(texture.sample_count() > 1, MS, "Texture multisample state ({}) does not match TexSampled<.., MS>", texture.sample_count() > 1);
        assert!(texture.usage().contains(kind::TexSampled::<F, D, MS>::usage()), "Texture usage ({:?}) does not contain TexSampled<F, D, MS>: {:?}", texture.usage(), kind::TexSampled::<F, D, MS>::usage());
    }
}

impl<F: fmt::TexFormat, A: access::StorageAccess, D: dim::Dimension> BindableTexture<kind::TexStorage<F, A, D>> {
    /// Creates a [`BindableTexture`] with a specified size.
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
            format: Some(F::FORMAT),
            dimension: Some(D::VIEW_DIMENSION),
            ..Default::default()
        });

        Self {
            texture,
            view,
            _kind: std::marker::PhantomData,
        }
    }

    /// Creates a [`BindableTexture`] from an existing [`wgpu::Texture`].
    /// 
    /// Note that this will **panic** if the supplied texture does not match the [`BindableTexture`]'s type signature.
    pub fn from_storage(texture: &wgpu::Texture) -> Self {
        Self::check_validity(texture);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(F::FORMAT),
            dimension: Some(D::VIEW_DIMENSION),
            ..Default::default()
        });

        Self { texture: texture.clone(), view, _kind: std::marker::PhantomData, }
    }

    /// Updates the underlying [`wgpu::Texture`] using a &\[u8\] as pixel data.
    /// 
    /// Note that this will **panic** if the supplied pixel data does not match the number of bytes
    /// the [`BindableTexture`] expects.
    pub fn update(&self, queue: &wgpu::Queue, size: wgpu::Extent3d, data: &[u8]) {
        let expected_bytes = F::BYTES_PER_PIXEL * size.width * size.height * size.depth_or_array_layers;
        assert_eq!(
            data.len() as u32, expected_bytes,
            "Data length does not match expected size for this BindableTexture\n\nExpected: {}\nReceived: {}", data.len(), expected_bytes  
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.width * F::BYTES_PER_PIXEL),
                rows_per_image: Some(size.height)
            },
            size
        );
    }

    // Assert validity
    fn check_validity(texture: &wgpu::Texture) {
        assert_eq!(texture.format(), F::FORMAT, "Texture format ({:?}) does not match TexStorage<F, ..>", texture.format());
        assert_eq!(texture.dimension(), D::physical_dimension(), "Texture dimension ({:?}) does not match TexStorage<.., D>", texture.dimension());
        assert!(texture.usage().contains(kind::TexStorage::<F, A, D>::usage()), "Texture usage ({:?}) does not contain TexStorage<F, A, D> ({:?})", texture.usage(), kind::TexStorage::<F, A, D>::usage());
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

/// The implementation of [BindableField] for an array of [`wgpu::Texture`] binding resources.
pub struct BindableTextureArray<const MAX_TEXTURES: usize, Kind: kind::TextureKind> {
    textures: [wgpu::Texture; MAX_TEXTURES],
    views: Box<[wgpu::TextureView; MAX_TEXTURES]>, // SAFETY: This CANNOT be reassigned once initialized!
    view_refs: UnsafeCell<[&'static wgpu::TextureView; MAX_TEXTURES]>,
    _kind: std::marker::PhantomData<Kind>
}

impl<const MAX_TEXTURES: usize, F: fmt::TexFormat, D: dim::Dimension, const MS: bool>
    BindableTextureArray<MAX_TEXTURES, kind::TexSampled<F, D, MS>> {
    /// Creates a [`BindableTextureArray`] where each [`wgpu::Texture`] has the specified size.
    pub fn new_sampled(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        label: Option<&str>
    ) -> Self {
        let mut textures  = Vec::with_capacity(MAX_TEXTURES);
        let mut views_vec = Vec::with_capacity(MAX_TEXTURES);
        for _ in 0..MAX_TEXTURES {
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

        let views: Box<[wgpu::TextureView; MAX_TEXTURES]> = Box::new(views_vec.try_into().unwrap());
        let refs: [&wgpu::TextureView; MAX_TEXTURES] = views.each_ref();
        let refs_static: [&'static wgpu::TextureView; MAX_TEXTURES] =
            unsafe { std::mem::transmute(refs) };

        Self {
            textures: textures.try_into().unwrap(),
            views,
            view_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }

    /// Creates a [`BindableTextureArray`] from an existing [`wgpu::Texture`] vector.
    /// 
    /// Note that this will **panic** if any supplied texture does not match the [`BindableTexture`]'s type signature,
    /// or if the supplied [`wgpu::Texture`] array is longer than MAX_TEXTURES.
    pub fn from_sampled(device: &wgpu::Device, textures: &Vec<wgpu::Texture>) -> Self {
        assert!(textures.len() <= MAX_TEXTURES,
            "Failed to create BindableTextureArray: provided texture array length ({}) was greater than MAX_TEXTURES ({MAX_TEXTURES}).", textures.len());
        
        // Using BindableTexture (singular) as a proxy to do the type checking automatically.
        // Since BindableTexture is a Zero-Cost-Abstraction, it makes sense to use it. 
        let mut textures_vec: Vec<wgpu::Texture> = vec![];
        let mut views_vec = vec![];
        for t in textures {
            let bt = BindableTexture::<kind::TexSampled<F, D, MS>>::from_sampled(t);
            textures_vec.push(bt.texture.clone());
            views_vec.push(bt.view);
        }

        for _ in textures.len()..MAX_TEXTURES {
            let bt = BindableTexture::<kind::TexSampled<F, D, MS>>::new_sampled(device, wgpu::Extent3d::default(), None);
            textures_vec.push(bt.texture.clone());
            views_vec.push(bt.view);
        }

        let views: Box<[wgpu::TextureView; MAX_TEXTURES]> = Box::new(views_vec.try_into().unwrap());
        let refs: [&wgpu::TextureView; MAX_TEXTURES] = views.each_ref();
        let refs_static: [&'static wgpu::TextureView; MAX_TEXTURES] =
        unsafe { std::mem::transmute(refs) };

        Self {
            textures: textures_vec.try_into().unwrap(),
            views,
            view_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }

    /// Sets a [`BindableTexture`] at index i using an existing [`wgpu::Texture`].
    /// 
    /// Note that this will **panic** if the supplied texture does not match the [`BindableTexture`]'s type signature.
    pub fn set_texture(&mut self, i: usize, texture: &wgpu::Texture) {
        let typed_texture: BindableTexture<kind::TexSampled<F, D, MS>> = BindableTexture::from_sampled(texture);
        self.textures[i] = typed_texture.texture;
        self.views[i] = typed_texture.view;
    }
}

impl<const MAX_TEXTURES: usize, F: fmt::TexFormat, A: access::StorageAccess, D: dim::Dimension>
    BindableTextureArray<MAX_TEXTURES, kind::TexStorage<F, A, D>> {
    /// Creates a [`BindableTextureArray`] where each [`wgpu::Texture`] has the specified size.
    pub fn new_storage(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        label: Option<&str>
    ) -> Self {
        let mut textures  = Vec::with_capacity(MAX_TEXTURES);
        let mut views_vec = Vec::with_capacity(MAX_TEXTURES);
        for _ in 0..MAX_TEXTURES {
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

        let views: Box<[wgpu::TextureView; MAX_TEXTURES]> = Box::new(views_vec.try_into().unwrap());
        let refs: [&wgpu::TextureView; MAX_TEXTURES] = views.each_ref();
        let refs_static: [&'static wgpu::TextureView; MAX_TEXTURES] =
            unsafe { std::mem::transmute(refs) };

        Self {
            textures: textures.try_into().unwrap(),
            views,
            view_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }

    /// Creates a [`BindableTextureArray`] from an existing [`wgpu::Texture`] vector.
    /// 
    /// Note that this will **panic** if any supplied texture does not match the [`BindableTexture`]'s type signature,
    /// or if the supplied [`wgpu::Texture`] array is longer than MAX_TEXTURES.
    pub fn from_storage(device: &wgpu::Device, textures: &Vec<wgpu::Texture>) -> Self {
        assert!(textures.len() <= MAX_TEXTURES,
            "Failed to create BindableTextureArray: provided texture array length ({}) was greater than MAX_TEXTURES ({MAX_TEXTURES}).", textures.len());
        
        // Using BindableTexture (singular) as a proxy to do the type checking automatically.
        // Since BindableTexture is a Zero-Cost-Abstraction, it makes sense to use it. 
        let mut textures_vec: Vec<wgpu::Texture> = vec![];
        let mut views_vec = vec![];
        for t in textures {
            let bt = BindableTexture::<kind::TexStorage<F, A, D>>::from_storage(t);
            textures_vec.push(bt.texture.clone());
            views_vec.push(bt.view);
        }

        for _ in textures.len()..MAX_TEXTURES {
            let bt = BindableTexture::<kind::TexStorage<F, A, D>>::new_storage(device, wgpu::Extent3d::default(), None);
            textures_vec.push(bt.texture.clone());
            views_vec.push(bt.view);
        }

        let views: Box<[wgpu::TextureView; MAX_TEXTURES]> = Box::new(views_vec.try_into().unwrap());
        let refs: [&wgpu::TextureView; MAX_TEXTURES] = views.each_ref();
        let refs_static: [&'static wgpu::TextureView; MAX_TEXTURES] =
        unsafe { std::mem::transmute(refs) };

        Self {
            textures: textures_vec.try_into().unwrap(),
            views,
            view_refs: UnsafeCell::new(refs_static),
            _kind: std::marker::PhantomData,
        }
    }

    /// Sets a [`BindableTexture`] at index i using an existing [`wgpu::Texture`].
    /// 
    /// Note that this will **panic** if the supplied texture does not match the [`BindableTexture`]'s type signature.
    pub fn set_texture(&mut self, i: usize, texture: &wgpu::Texture) {
        let typed_texture: BindableTexture<kind::TexStorage<F, A, D>> = BindableTexture::from_storage(texture);
        self.textures[i] = typed_texture.texture;
        self.views[i] = typed_texture.view;
    }
}

impl<const MAX_TEXTURES: usize, Kind: kind::TextureKind> Index<usize> for BindableTextureArray<MAX_TEXTURES, Kind> {
    type Output = wgpu::Texture;

    fn index(&self, index: usize) -> &Self::Output {
        &self.textures[index]
    }
}

impl<const MAX_TEXTURES: usize, Kind: kind::TextureKind> BindableField for BindableTextureArray<MAX_TEXTURES, Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: Kind::binding_type(),
            count: std::num::NonZeroU32::new(MAX_TEXTURES as u32),
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        let refs: [&wgpu::TextureView; MAX_TEXTURES] = self.views.each_ref();

        unsafe {
            *self.view_refs.get() = std::mem::transmute(refs);

            wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureViewArray(&*self.view_refs.get()),
            }   
        }
    }
}