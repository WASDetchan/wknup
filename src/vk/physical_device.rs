use std::{error::Error, sync::Arc, vec::IntoIter};

use ash::vk::{PhysicalDevice, PhysicalDeviceType, QueueFamilyProperties, QueueFlags, SurfaceKHR};

use super::instance::InstanceManager;

type QFFilter = Arc<dyn Fn(PhysicalDevice, usize, &QueueFamilyProperties) -> bool>;
#[derive(Clone)]
pub struct QueueFamilyIndices {
    pub graphics: Option<usize>,
    pub present: Option<usize>,
    graphics_filter: QFFilter,
    present_filter: QFFilter,
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
            Arc::new(move |_device, _id, props| props.queue_flags.contains(QueueFlags::GRAPHICS)),
            Arc::new(move |device, id, _props| {
                instance
                    .get_physical_device_surface_support(device, id as u32, surface_khr)
                    .unwrap_or(false)
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
    qfi.fill(instance, device);
    ((props.device_type == PhysicalDeviceType::DISCRETE_GPU
        || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
        && (features.geometry_shader == 1)
        && qfi.is_complete()) as i32
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
