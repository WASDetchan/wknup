use ash::{
    Instance,
    vk::{self, PhysicalDeviceFeatures, PhysicalDeviceVulkanMemoryModelFeatures},
};

#[derive(Default, Debug)]
pub struct FeaturesInfo {
    pub features: PhysicalDeviceFeatures,
    pub vulkan_memory_model: bool,
    pub vulkan_memory_model_device_scope: bool,
    pub vulkan_memory_model_availability_visibility_chains: bool,
}

impl FeaturesInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_features2(features2: PhysicalDeviceFeatures2) -> Self {
        let mut s = Self::default();
        let vulkan_memory_model_features = features2.vulkan_memory_model_features;
        let features2 = features2.features2;
        s.features = features2.features;
        s.vulkan_memory_model = vulkan_memory_model_features.vulkan_memory_model > 0;
        s.vulkan_memory_model_device_scope =
            vulkan_memory_model_features.vulkan_memory_model_device_scope > 0;
        s.vulkan_memory_model_availability_visibility_chains =
            vulkan_memory_model_features.vulkan_memory_model_availability_visibility_chains > 0;
        s
    }
}

#[derive(Debug)]
pub struct PhysicalDeviceFeatures2<'a> {
    features2: vk::PhysicalDeviceFeatures2<'a>,
    vulkan_memory_model_features: PhysicalDeviceVulkanMemoryModelFeatures<'a>,
}

impl<'a> PhysicalDeviceFeatures2<'a> {
    pub fn new() -> Self {
        let mut s = Self {
            features2: vk::PhysicalDeviceFeatures2::default(),
            vulkan_memory_model_features: PhysicalDeviceVulkanMemoryModelFeatures::default(),
        };
        s.features2
            .push_next::<PhysicalDeviceVulkanMemoryModelFeatures>(
                &mut s.vulkan_memory_model_features,
            );
        let next_ptr: *mut PhysicalDeviceVulkanMemoryModelFeatures =
            <*mut PhysicalDeviceVulkanMemoryModelFeatures>::cast(
                &mut s.vulkan_memory_model_features,
            );
        s.features2.p_next = next_ptr as _;
        s
    }

    pub unsafe fn fill(&mut self, device: vk::PhysicalDevice, instance: &Instance) {
        instance.get_physical_device_features2(device, &mut self.features2);
    }
}
