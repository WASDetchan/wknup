use std::sync::Arc;

use ash::vk;

use super::Device;

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
}
