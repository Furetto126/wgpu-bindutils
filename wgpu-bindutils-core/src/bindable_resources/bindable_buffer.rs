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

pub struct BindableBuffer<T: NoUninit, Kind = BufUniform> {
    pub value: T,
    buffer: wgpu::Buffer,
    _kind: std::marker::PhantomData<Kind>
}

impl<T: bytemuck::NoUninit, Kind: BufferKind> BindableBuffer<T, Kind> {
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

    pub fn update(&mut self, queue: &wgpu::Queue, value: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[value]));
        self.value = value;
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

pub struct BindableBufferVector<T: NoUninit, Kind = BufUniform> {
    pub value: Vec<T>,
    buffer: wgpu::Buffer,
    _kind: std::marker::PhantomData<Kind>
}

impl<T: bytemuck::NoUninit, Kind: BufferKind> BindableBufferVector<T, Kind> {
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

    pub fn update(&mut self, queue: &wgpu::Queue, value: Vec<T>) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&value));
        self.value = value;
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
