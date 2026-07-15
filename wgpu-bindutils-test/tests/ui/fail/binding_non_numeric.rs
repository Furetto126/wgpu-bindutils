#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
#[visibility(ShaderStages::COMPUTE)]
struct Test {
    #[binding("NaN")]
    t: BindableBuffer<f32>
}

fn main() {}