pub mod features;

use std::sync::Arc;

use ash::vk::{PhysicalDevice, PhysicalDeviceType};

use crate::vk::{
    device::{self, device_extensions, queues::QueueFamilySelector},
    error::fatal_vk_error,
    instance::Instance,
};

fn rate_physical_device<T: QueueFamilySelector>(
    instance: &Arc<Instance>,
    device: PhysicalDevice,
    mut qfc: T,
) -> PhysicalDeviceChoice<T> {
    let info = unsafe { instance.get_physical_device_info(device) };
    let props = info.properties;
    let features = info.features;
    let mut queue_counts = Vec::new();

    if !(props.device_type == PhysicalDeviceType::DISCRETE_GPU
        || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
    {
        return PhysicalDeviceChoice {
            rating: 0,
            queue_counts,
            device,
            queue_family_selector: qfc,
        };
    }

    if device_extensions::check_extensions(instance, device, &device::REQUIRED_DEVICE_EXTENSIONS)
        .is_err()
    {
        return PhysicalDeviceChoice {
            rating: 0,
            queue_counts,
            device,
            queue_family_selector: qfc,
        };
    }

    if features.check_required().is_err() {
        return PhysicalDeviceChoice {
            rating: 0,
            queue_counts,
            device,
            queue_family_selector: qfc,
        };
    }

    unsafe { instance.get_physical_device_queue_family_properties(device) }
        .into_iter()
        .enumerate()
        .for_each(|(id, prop)| {
            queue_counts.push(prop.queue_count);
            qfc.inspect_queue_family(device, id.try_into().unwrap(), prop)
        });

    if !qfc.is_complete() {
        return PhysicalDeviceChoice {
            rating: 0,
            queue_counts,
            device,
            queue_family_selector: qfc,
        };
    }

    return PhysicalDeviceChoice {
        rating: 1,
        queue_counts,
        device,
        queue_family_selector: qfc,
    };
}

#[derive(Debug, thiserror::Error)]
pub enum PhysicalDeviceChoiceError {
    #[error("no physical device found")]
    DeviceNotFound,
    #[error("no suitable physical device found")]
    SuitableDeviceNotFound,
}

#[derive(Clone)]
pub struct PhysicalDeviceChoice<T: QueueFamilySelector> {
    rating: i32,
    pub device: PhysicalDevice,
    pub queue_family_selector: T,
    pub queue_counts: Vec<u32>,
}
pub fn choose_physical_device<T: QueueFamilySelector>(
    instance: &Arc<Instance>,
    queue_family_selector: T,
) -> Result<PhysicalDeviceChoice<T>, PhysicalDeviceChoiceError> {
    let Some(physical_device_choice) = instance
        .enumerate_physical_devices()
        .unwrap_or_else(|e| fatal_vk_error("failed to enumerate_physical_devices", e))
        .into_iter()
        .map(|device| rate_physical_device(instance, device, queue_family_selector.clone()))
        .max_by_key(|s| s.rating)
    else {
        return Err(PhysicalDeviceChoiceError::DeviceNotFound);
    };

    if physical_device_choice.rating <= 0 {
        return Err(PhysicalDeviceChoiceError::SuitableDeviceNotFound);
    }

    Ok(physical_device_choice)
}
