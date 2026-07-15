#![allow(warnings)]

use wgpu_bindutils::prelude::*;
use wgpu::ShaderStages;

#[derive(BindableStruct)]
struct Test {
    #[visibility(ShaderStages::FRAGMENT)]
    t: f32
}

fn main() {}