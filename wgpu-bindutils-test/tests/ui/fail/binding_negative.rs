#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
#[visibility(ShaderStages::COMPUTE)]
struct Test {
    #[binding(-1i32)]
    t: BindableBuffer<f32>
}

fn main() {}