use std::sync::Arc;

use ash::vk;

use super::Device;

use crate::vk::{
    command_buffer::CommandBuffer, error::fatal_vk_error, fence::Fence, semaphore::Semaphore,
    swapchain::Swapchain,
};

pub trait QueueFamilySelector: Clone {
    type Q: Queues;
    fn inspect_queue_family(
        &mut self,
        physical_device: vk::PhysicalDevice,
        queue_family_id: u32,
        queue_family_properties: vk::QueueFamilyProperties,
    );

    fn is_complete(&self) -> bool;

    fn requirements(&self) -> Vec<(u32, Vec<f32>)>;

    fn fill_queues(&self, queues_raw: Vec<(u32, Vec<Queue>)>) -> Self::Q;
}

pub trait Queues {}

#[derive(Clone)]
pub struct Queue {
    device: Arc<Device>,
    queue: Arc<vk::Queue>,
}

impl Queue {
    pub fn new(device: Arc<Device>, queue: vk::Queue) -> Self {
        Self {
            device,
            queue: Arc::new(queue),
        }
    }

    pub fn submit_command_buffer(
        &self,
        command_buffer: Arc<CommandBuffer>,
        wait: &[&Semaphore],
        signal: &[&Semaphore],
        wait_mask: &[vk::PipelineStageFlags],
        fence: Option<&mut Fence>,
    ) {
        let wait: Vec<_> = wait
            .into_iter()
            .map(|s| unsafe { s.raw_handle() })
            .collect();
        let signal: Vec<_> = signal
            .into_iter()
            .map(|s| unsafe { s.raw_handle() })
            .collect();
        let cbs = [unsafe { command_buffer.raw_handle() }];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait)
            .signal_semaphores(&signal)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&cbs);

        let fence = if let Some(fence) = fence {
            unsafe {
                fence.reset();
                fence.raw_handle()
            }
        } else {
            vk::Fence::null()
        };

        unsafe {
            self.device
                .raw_handle()
                .queue_submit(self.queue.as_ref().clone(), &[submit_info], fence)
                .unwrap_or_else(|error| fatal_vk_error("failed t osubmit queue", error));
        }
    }

    pub fn present(&self, swapchain: &Swapchain, index: u32, wait: &[&Semaphore]) {
        let wait: Vec<_> = wait
            .into_iter()
            .map(|s| unsafe { s.raw_handle() })
            .collect();

        let swapchain_khr = [unsafe { swapchain.raw_handle() }];

        let index = [index];

        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchain_khr)
            .wait_semaphores(&wait)
            .image_indices(&index);

        unsafe {
            swapchain
                .device_handle()
                .queue_present(self.queue.as_ref().clone(), &present_info)
                .unwrap_or_else(|error| fatal_vk_error("failed t osubmit queue", error));
        }
    }
}
