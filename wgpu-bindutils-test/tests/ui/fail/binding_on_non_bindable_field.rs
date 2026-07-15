#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
#[visibility(ShaderStages::COMPUTE)]
struct Test {
    #[binding(0)]
    t: f32
}

fn main() {}