use std::sync::Arc;

use ash::vk;

use super::{
    device::queues::{Queue, QueueFamilySelector, Queues},
    instance::Instance,
    surface::Surface,
    swapchain,
};

pub struct DrawQueues {
    pub graphics: Queue,
    pub present: Queue,
}

impl Queues for DrawQueues {}

#[derive(Clone)]
pub struct DrawQueueFamilySelector {
    _instance: Arc<Instance>,
    surface: Arc<Surface>,
    pub graphics: Option<u32>,
    pub present: Option<u32>,
}

impl DrawQueueFamilySelector {
    pub fn new(instance: Arc<Instance>, surface: Arc<Surface>) -> Self {
        Self {
            _instance: instance,
            surface,
            graphics: None,
            present: None,
        }
    }
    fn filter_present_qf(
        &self,
        device: vk::PhysicalDevice,
        id: u32,
        _props: vk::QueueFamilyProperties,
    ) -> bool {
        let support = self.surface.get_physical_device_surface_support(device, id);
        if !support.is_ok_and(|s| s) {
            return false;
        }

        let Ok(surface_info) = self.surface.get_physical_device_surface_info(device) else {
            return false;
        };
        if !swapchain::check_surface_info(surface_info) {
            return false;
        }
        true
    }

    fn filter_graphic_qf(
        &self,
        _device: vk::PhysicalDevice,
        _id: u32,
        props: vk::QueueFamilyProperties,
    ) -> bool {
        props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
    }
}

impl QueueFamilySelector for DrawQueueFamilySelector {
    type Q = DrawQueues;
    fn inspect_queue_family(
        &mut self,
        physical_device: vk::PhysicalDevice,
        queue_family_id: u32,
        queue_family_properties: vk::QueueFamilyProperties,
    ) {
        if self.filter_graphic_qf(physical_device, queue_family_id, queue_family_properties) {
            self.graphics = Some(queue_family_id);
        };
        if self.filter_present_qf(physical_device, queue_family_id, queue_family_properties) {
            self.present = Some(queue_family_id);
        }
    }

    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }

    fn requirements(&self) -> Vec<(u32, Vec<f32>)> {
        if !self.is_complete() {
            panic!("asked for requirements of an unscompleted chooser!");
        }

        let g = self.graphics.unwrap();
        let p = self.present.unwrap();

        if g == p {
            return vec![(g, vec![0.0f32])];
        } else {
            return vec![(g, vec![0.0f32]), (p, vec![0.0f32])];
        }
    }

    fn fill_queues(&self, queues_raw: Vec<(u32, Vec<Queue>)>) -> DrawQueues {
        if !self.is_complete() {
            panic!("filled queues of an unscompleted chooser!");
        }
        let g = self.graphics.unwrap();
        let p = self.present.unwrap();

        DrawQueues {
            present: queues_raw.iter().find(|(id, _queues)| *id == p).unwrap().1[0].clone(),
            graphics: queues_raw.iter().find(|(id, _queues)| *id == g).unwrap().1[0].clone(),
        }
    }
}
