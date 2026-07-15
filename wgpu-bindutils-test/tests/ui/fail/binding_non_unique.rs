#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
#[visibility(ShaderStages::COMPUTE)]
struct Test {
    #[binding(0)]
    t1: BindableBuffer<f32>,
    #[binding(0)]
    t2: BindableBuffer<u64>
}

fn main() {}