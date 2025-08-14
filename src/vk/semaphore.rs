use std::sync::Arc;

use ash::vk;

use super::{device::Device, error::fatal_vk_error};

pub struct Semaphore {
    device: Arc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Arc<Device>) -> Self {
        let create_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe {
            device
                .raw_handle()
                .create_semaphore(&create_info, None)
                .unwrap_or_else(|error| fatal_vk_error("failed to create_semaphore", error))
        };
        Self { device, semaphore }
    }

    pub(in crate::vk) unsafe fn raw_handle(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw_handle()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}
