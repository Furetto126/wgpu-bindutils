#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
struct Test {
    #[binding(0)]
    #[visibility("Not a ShaderStage")]
    t: BindableBuffer<f32>
}

fn main() {}