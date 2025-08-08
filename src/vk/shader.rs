use std::sync::Arc;

use ash::vk;

use super::device::DeviceManager;

pub struct ShaderModule {
    device: Arc<DeviceManager>,
    shader: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(device: Arc<DeviceManager>, shader_raw: &[u32]) -> Self {
        Self {
            shader: unsafe { device.create_shader_module(shader_raw).unwrap() },
            device,
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.shader).unwrap();
        }
    }
}
