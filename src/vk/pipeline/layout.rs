use std::sync::Arc;

use ash::vk;

use crate::vk::device::Device;

pub struct PipelineLayout {
    device: Arc<Device>,
    layout: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(device: Arc<Device>) -> Self {
        let layout_info = vk::PipelineLayoutCreateInfo::default();
        let layout = unsafe { device.create_pipeline_layout(layout_info) };

        Self { device, layout }
    }

    pub unsafe fn raw_handle(&self) -> vk::PipelineLayout {
        self.layout
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline_layout(self.layout);
        }
    }
}
