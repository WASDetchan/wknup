pub mod features;

use std::{error::Error, sync::Arc, vec::IntoIter};

use ash::vk::{
    PhysicalDevice, PhysicalDeviceType, PresentModeKHR, QueueFamilyProperties, QueueFlags,
    SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR,
};

use super::{
    device::{self, device_extensions, swapchain},
    instance::InstanceManager,
};

type QFFilter = Arc<dyn Fn(PhysicalDevice, usize, &QueueFamilyProperties) -> bool + Send + Sync>;
#[derive(Clone)]
pub struct QueueFamilyIndices {
    pub graphics: Option<usize>,
    pub present: Option<usize>,
    graphics_filter: QFFilter,
    present_filter: QFFilter,
}

fn filter_present_qf(
    instance: &Arc<InstanceManager>,
    surface: SurfaceKHR,
    device: PhysicalDevice,
    id: usize,
    _props: &QueueFamilyProperties,
) -> bool {
    let support = instance.get_physical_device_surface_support(device, id as u32, surface);
    if !support.is_ok_and(|s| s) {
        return false;
    }

    let surface_info = query_device_surface_info(instance, device, surface).unwrap();
    if !swapchain::check_surface_info(surface_info) {
        return false;
    }
    true
}

fn filter_graphic_qf(_device: PhysicalDevice, _id: usize, props: &QueueFamilyProperties) -> bool {
    props.queue_flags.contains(QueueFlags::GRAPHICS)
}

impl QueueFamilyIndices {
    fn new(graphics_filter: QFFilter, present_filter: QFFilter) -> Self {
        Self {
            graphics: None,
            present: None,
            graphics_filter,
            present_filter,
        }
    }
    fn try_queue(
        &mut self,
        physical_device: PhysicalDevice,
        id: usize,
        props: &QueueFamilyProperties,
    ) {
        if (self.graphics_filter)(physical_device, id, props) {
            self.graphics = Some(id);
        };
        if (self.present_filter)(physical_device, id, props) {
            self.present = Some(id);
        }
    }
    fn fill(&mut self, instance: &Arc<InstanceManager>, physical_device: PhysicalDevice) {
        Self::iterate_physical_device_queue_families(instance, physical_device)
            .enumerate()
            .for_each(|(id, prop)| self.try_queue(physical_device, id, &prop));
    }
    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
    fn iterate_physical_device_queue_families(
        instance: &Arc<InstanceManager>,
        physical_device: PhysicalDevice,
    ) -> IntoIter<QueueFamilyProperties> {
        instance
            .get_physical_device_queue_family_properties(physical_device)
            .unwrap()
            .into_iter()
    }
    pub fn default(instance: Arc<InstanceManager>, surface_khr: SurfaceKHR) -> QueueFamilyIndices {
        QueueFamilyIndices::new(
            Arc::new(filter_graphic_qf),
            Arc::new(move |device, id, props| {
                filter_present_qf(&instance, surface_khr, device, id, props)
            }),
        )
    }
}
fn rate_physical_device(
    instance: &Arc<InstanceManager>,
    device: PhysicalDevice,
    mut qfi: QueueFamilyIndices,
) -> i32 {
    let info = instance.get_physical_device_info(device).unwrap();
    let props = info.properties;
    let features = info.features;

    if !(props.device_type == PhysicalDeviceType::DISCRETE_GPU
        || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
    {
        return 0;
    }

    if device_extensions::check_extensions(instance, device, &device::REQUIRED_DEVICE_EXTENSIONS)
        .is_err()
    {
        return 0;
    }

    if !features.vulkan_memory_model {
        return 0;
    }

    if features.features.geometry_shader != 1 {
        return 0;
    }
    qfi.fill(instance, device);
    if !qfi.is_complete() {
        return 0;
    }

    1
}

fn iterate_physical_devices(
    instance: &Arc<InstanceManager>,
) -> Result<IntoIter<PhysicalDevice>, Box<dyn Error>> {
    Ok(instance.enumerate_physical_devices()?.into_iter())
}

pub struct PhysicalDeviceChoice {
    pub device: PhysicalDevice,
    pub queue_family_indices: QueueFamilyIndices,
}
pub fn choose_physical_device(
    instance: &Arc<InstanceManager>,
    mut queue_family_indices: QueueFamilyIndices,
) -> Result<PhysicalDeviceChoice, Box<dyn Error>> {
    let physical_device = iterate_physical_devices(instance)?
        .map(|pdev| {
            (
                rate_physical_device(instance, pdev, queue_family_indices.clone()),
                pdev,
            )
        })
        .max_by_key(|s| s.0);
    let Some(physical_device) = physical_device else {
        return Err("No physical device found.".into());
    };
    if physical_device.0 <= 0 {
        return Err("No suitable physical device found.".into());
    }
    let physical_device = physical_device.1;
    queue_family_indices.fill(instance, physical_device);
    Ok(PhysicalDeviceChoice {
        device: physical_device,
        queue_family_indices,
    })
}

pub fn query_device_surface_info(
    instance: &Arc<InstanceManager>,
    device: PhysicalDevice,
    surface: SurfaceKHR,
) -> Result<PhysicalDeviceSurfaceInfo, Box<dyn Error>> {
    instance.get_physical_device_surface_info(device, surface)
}

pub struct PhysicalDeviceSurfaceInfo {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}
