# wgpu-bindutils
[![Rust](https://github.com/Furetto126/wgpu-bindutils/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/Furetto126/wgpu-bindutils/actions/workflows/rust.yml)

A wgpu wrapper around `BindGroup` and `BindGroupLayout` focused on type-safety and ease of use.

## The Problem

It is usual to structure (pun intended) a `BindGroup` as a struct which holds the various buffers, textures, etc.

This, however, can become quickly cumbersome as the number of bindings in the `BindGroup` increases.

Projects which benefit from explicit `BindGroupLayout`s will also find each "bind-struct" to have duplicated information, such as bindings and data types, between the definition of the `BindGroupLayout` and `BindGroup`, which can be difficult to track and maintain, and can lead to easy-to-miss errors and mismatches.

## How wgpu-bindutils Works

`wgpu-bindutils` encodes a struct as a single `BindGroup`, with each field being a single resource (or `BindableField`).

The `BindableField` trait has already been implemented on some custom structs which hold, in a type-safe way, the various possible options needed to define the resource (such as `TextureFormat` for a texture, `SamplerBindingType` for a sampler, ...).

Each `BindableField` exposes two methods:
- a static one that returns a `BindGroupLayoutEntry` describing the "shape" of the data
- another which uses the actual data to construct the needed resource and returns a `BindGroupEntry`

A `BindableStruct` will then cycle through its fields and build the actual `BindGroupLayout` and `BindGroup`.

To avoid having to write the various different implementations every time, a convenience derive macro is included with its helper attributes.

## How to Use

Here is a sample `BindableStruct` implementation which uses the various different features of `wgpu-bindutils` (before getting scared of the generics, keep reading):

```rust
use wgpu_bindutils::prelude::*;

#[derive(BindableStruct)]
#[visibility(ShaderStages::COMPUTE)]
struct MyBindGroup {
    #[binding(0)]
    #[visibility(ShaderStages::VERTEX | ShaderStages::FRAGMENT)]
    pub buffer: BindableBufferVector<f32, buf_kind::BufStorage>,

    #[binding(1)]
    pub texture: BindableTexture<tex_opts::kind::TexStorage<tex_opts::fmt::Rgba32Float, tex_opts::access::ReadOnly, tex_opts::dim::D2>>,

    #[binding(2)]
    pub sampler: BindableSampler<samp_kind::NonFiltering>,

    #[binding(3)]
    pub texture_array: BindableTextureArray<5, tex_opts::kind::TexSampled<tex_opts::fmt::Rgba32Float, tex_opts::dim::D2, false>>,

    #[binding(4)]
    pub sampler_array: BindableSamplerArray<5>,

    pub something_else_entirely: f32,
}
```

### How to Read `MyBindGroup`

First we use the derive macro to auto-derive `BindableStruct` for us.

Second, we (optionally) define a default visibility for the fields that otherwise don't have one specified; if a field's visibility conflicts with the default one, the former will be used.

Then we use our `BindableField`s, which come pre-packaged as:
`BindableBuffer`, `BindableBufferVector`, `BindableTexture`, `BindableTextureArray`, `BindableSampler`, `BindableSamplerArray`.

Each of them has options that can be specified at compile time via generics.

- Every `BindableField` **must** have a binding specified; other fields **must not** have one specified (checked at compile time).
- If no default visibility was provided, every `BindableField` **must** specify one; other fields **must not** specify one (checked at compile time).

To then use `MyBindGroup` in practice, we simply instantiate it, for example, as such:

```rust
let mut my_group = MyBindGroup {
    buffer: BindableBufferVector::new(&device, vec![0.0, 1.0, 2.0]),
    texture: BindableTexture::new_storage(
        &device,
        wgpu::Extent3d {
            width: 10,
            height: 67,
            depth_or_array_layers: 1,
        },
        Some("texture"),
    ),
    sampler: BindableSampler::new(
        &device,
        &wgpu::SamplerDescriptor::default(),
    ),
    texture_array: BindableTextureArray::new_sampled(
        &device,
        wgpu::Extent3d {
            width: 1,
            height: 2,
            depth_or_array_layers: 1,
        },
        Some("Texture array"),
    ),
    sampler_array: BindableSamplerArray::new(
        &device,
        &[
            &wgpu::SamplerDescriptor::default(),
            &wgpu::SamplerDescriptor::default(),
            &wgpu::SamplerDescriptor::default(),
            &wgpu::SamplerDescriptor::default(),
            &wgpu::SamplerDescriptor::default(),
        ],
    ),
    something_else_entirely: 0.0,
};

my_group.buffer.update(&queue, vec![3.0, 4.0, 5.0]);

my_group.texture_array.set_texture(
    0,
    BindableTexture::new_sampled(
        &device,
        wgpu::Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
        Some("New texture in array!"),
    ),
);

let layout = MyBindGroup::bind_group_layout(&device);
let bind_group = my_group.bind_group(&device);
```

Note how, thanks to type-safety, we don't have to manually specify options such as texture formats every time we perform operations on a `BindableTexture` — the same logic applies to all other `BindableField`s.

As soon as we have our `BindableStruct` defined, we can get its `BindGroupLayout` without ever having instantiated it. Once we have an instance, we can also get its `BindGroup`, ready to be plugged into a shader pass.
