use std::sync::{Mutex, OnceLock};

use wgpu::{Device, Queue};

struct GpuContext {
    device: Device,
    queue: Queue
}

static GPU: OnceLock<Mutex<GpuContext>> = OnceLock::new();

impl GpuContext {
    pub fn get() -> &'static Mutex<GpuContext> {
        GPU.get_or_init(|| {
            pollster::block_on(async {
                let instance = wgpu::Instance::default();
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::None,
                        force_fallback_adapter: true,
                        ..Default::default()
                    })
                    .await
                    .expect("No adapter found.");
                
                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor::default())
                    .await
                    .expect("Failed to create device.");

                Mutex::new(GpuContext { device, queue })
            })
        })
    }
}

pub fn panic_validation<F: FnMut() -> ()>(device: &wgpu::Device, f: &mut F) {
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    f();
    let error = pollster::block_on(error_scope.pop());
    assert!(error.is_none());
}