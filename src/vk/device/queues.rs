use ash::vk;

pub trait QueueFamilyChooser: Clone {
    type Q: Queues;
    fn inspect_queue_family(
        &mut self,
        physical_device: vk::PhysicalDevice,
        queue_family_id: u32,
        queue_family_properties: vk::QueueFamilyProperties,
    );

    fn is_complete(&self) -> bool;

    fn requirements(&self) -> Vec<(u32, Vec<f32>)>;

    fn fill_queues(&self, queues_raw: Vec<(u32, Vec<vk::Queue>)>) -> Self::Q;
}

pub trait Queues {}
