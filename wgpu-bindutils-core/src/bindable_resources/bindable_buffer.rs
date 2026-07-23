use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::bindable_resources::bindable::BindableField;

pub mod buf_kind {
    pub trait BufferKind {
        fn binding_type() -> wgpu::BufferBindingType;
        fn usage() -> wgpu::BufferUsages;
    }

    pub struct BufUniform;
    impl BufferKind for BufUniform {
        fn binding_type() -> wgpu::BufferBindingType {
            wgpu::BufferBindingType::Uniform
        }

        fn usage() -> wgpu::BufferUsages {
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        }
    }

    pub struct BufStorage<const READ_ONLY: bool = true>;
    impl<const READ_ONLY: bool> BufferKind for BufStorage<READ_ONLY> {
        fn binding_type() -> wgpu::BufferBindingType {
            wgpu::BufferBindingType::Storage { read_only: READ_ONLY }
        }

        fn usage() -> wgpu::BufferUsages {
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
        }
    }
}

use buf_kind::*;

/// The implementation of [BindableField] for a [wgpu::Buffer] binding resource of underlying type T.
pub struct BindableBuffer<T: NoUninit, Kind = BufUniform> {
    pub value: T,
    buffer: wgpu::Buffer,
    _kind: std::marker::PhantomData<Kind>
}

impl<T: bytemuck::NoUninit, Kind: BufferKind> BindableBuffer<T, Kind> {
    /// Create a [`BindableBuffer`] from a specified value.
    pub fn new(device: &wgpu::Device, value: T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[value]),
            usage: Kind::usage(),
        });

        Self {
            value,
            buffer,
            _kind: std::marker::PhantomData,
        }
    }

    /// Create a [`BindableBuffer`] from an existing [`wgpu::Buffer`].
    /// 
    /// Note that this will **panic** if the supplied buffer does not match the [`BindableBuffer`]'s type signature.<br>
    /// This function does not check that the provided current value is actually the one stored in the [`wgpu::Buffer`].
    pub fn from_buffer(buffer: &wgpu::Buffer, current_value: T) -> Self {
        Self::check_validity(buffer);
    
        Self {
            value: current_value,
            buffer: buffer.clone(),
            _kind: std::marker::PhantomData,
        }
    }

    /// Updates the underlying [`wgpu::Buffer`] using the supplied value.
    pub fn update(&mut self, queue: &wgpu::Queue, value: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[value]));
        self.value = value;
    }

    // Check validity
    fn check_validity(buffer: &wgpu::Buffer) {
        let expected_size = std::mem::size_of::<T>();
        assert_eq!(buffer.size(), expected_size as u64,
            "Buffer size ({}) does not match expected size of type `{}` ({})",
            buffer.size(), expected_size, std::any::type_name::<T>()
        );
        assert!(buffer.usage().contains(Kind::usage()), "Buffer usage ({:?}) does not contain Kind ({:?})", buffer.usage(), Kind::usage());
    }
}

impl<T: bytemuck::NoUninit, Kind: BufferKind> BindableField for BindableBuffer<T, Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: Kind::binding_type(),
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding()
        }
    }
}

/// The implementation of [BindableField] for a [wgpu::Buffer] binding resource of underlying type [`Vec<T>`].
pub struct BindableBufferVector<T: NoUninit, Kind = BufUniform> {
    pub value: Vec<T>,
    buffer: wgpu::Buffer,
    _kind: std::marker::PhantomData<Kind>
}

impl<T: bytemuck::NoUninit, Kind: BufferKind> BindableBufferVector<T, Kind> {
    /// Create a [`BindableBufferVector`] from a specified value.
    pub fn new(device: &wgpu::Device, value: Vec<T>) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&value),
            usage: Kind::usage(),
        });

        Self {
            value,
            buffer,
            _kind: std::marker::PhantomData,
        }
    }

    /// Create a [`BindableBufferVector`] from an existing [`wgpu::Buffer`].
    /// 
    /// Note that this will **panic** if the supplied buffer does not match the [`BindableBufferVector`]'s type signature and the supplied vector length.<br>
    /// This function does not check that the provided current value is actually the one stored in the [`wgpu::Buffer`].
    pub fn from_buffer(buffer: &wgpu::Buffer, current_value: Vec<T>) -> Self {
        Self::check_validity(buffer, current_value.len());
        Self {
            value: current_value,
            buffer: buffer.clone(),
            _kind: std::marker::PhantomData,
        }
    } 

    /// Updates the underlying [`wgpu::Buffer`] using the supplied value.
    pub fn update(&mut self, queue: &wgpu::Queue, value: Vec<T>) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&value));
        self.value = value;
    }

    // Check validity
    fn check_validity(buffer: &wgpu::Buffer, len: usize) {
        let expected_size = std::mem::size_of::<T>() * len;
        assert_eq!(buffer.size(), expected_size as u64,
            "Buffer size ({}) does not match expected size of type `Vec<{}>` ({})",
            buffer.size(), expected_size, std::any::type_name::<T>()
        );
        assert!(buffer.usage().contains(Kind::usage()), "Buffer usage ({:?}) does not contain Kind ({:?})", buffer.usage(), Kind::usage());
    }
}

impl<T: bytemuck::NoUninit, Kind: BufferKind> BindableField for BindableBufferVector<T, Kind> {
    fn layout_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: Kind::binding_type(),
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bind_group_entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding()
        }
    }
}
