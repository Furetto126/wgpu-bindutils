#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
struct Test {
    #[binding(0)]
    #[visibility((ShaderStages::VERTEX | ShaderStages::FRAGMENT | ShaderStages::COMPUTE) - ShaderStages::FRAGMENT)]
    t: BindableBuffer<f32>,
}

fn main() {}