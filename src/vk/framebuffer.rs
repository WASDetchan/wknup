use std::sync::Arc;

use ash::vk;

use super::{device::Device, pipeline::render_pass::RenderPass};

pub struct Framebuffer {
    device: Arc<Device>,
    _render_pass: Arc<RenderPass>,
    framebuffer: vk::Framebuffer,
}

impl Framebuffer {
    pub fn new(
        device: Arc<Device>,

        render_pass: Arc<RenderPass>,
        framebuffer: vk::Framebuffer,
    ) -> Self {
        Self {
            device,
            _render_pass: render_pass,
            framebuffer,
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(self.framebuffer);
        }
    }
}
