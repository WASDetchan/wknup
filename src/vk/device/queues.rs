pub mod family_chooser {
    use ash::vk;

    pub trait QueueFamilyChooser: Clone {
        fn inspect_queue_family(
            &mut self,
            physical_device: vk::PhysicalDevice,
            queue_family_id: u32,
            queue_family_properties: vk::QueueFamilyProperties,
        );

        fn is_complete(&self) -> bool;
    }
}
