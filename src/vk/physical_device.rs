pub mod features;

use std::sync::Arc;

use ash::vk::{self, PhysicalDevice, PhysicalDeviceType, QueueFamilyProperties, QueueFlags};

use super::{
    device::{self, device_extensions, queues::family_chooser::QueueFamilyChooser, swapchain},
    error::fatal_vk_error,
    instance::Instance,
    surface::SurfaceManager,
};

#[derive(Clone)]
pub struct Chooser {
    _instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
    pub graphics: Option<u32>,
    pub present: Option<u32>,
}

impl Chooser {
    pub fn new(instance: Arc<Instance>, surface: Arc<SurfaceManager>) -> Self {
        Self {
            _instance: instance,
            surface,
            graphics: None,
            present: None,
        }
    }
    fn filter_present_qf(
        &self,
        device: PhysicalDevice,
        id: u32,
        _props: QueueFamilyProperties,
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
        _device: PhysicalDevice,
        _id: u32,
        props: QueueFamilyProperties,
    ) -> bool {
        props.queue_flags.contains(QueueFlags::GRAPHICS)
    }
}

impl QueueFamilyChooser for Chooser {
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
}

fn rate_physical_device<QFC: QueueFamilyChooser>(
    instance: &Arc<Instance>,
    device: PhysicalDevice,
    mut qfc: QFC,
) -> PhysicalDeviceChoice<QFC> {
    let info = unsafe { instance.get_physical_device_info(device) };
    let props = info.properties;
    let features = info.features;

    if !(props.device_type == PhysicalDeviceType::DISCRETE_GPU
        || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
    {
        return PhysicalDeviceChoice {
            rating: 0,
            device,
            queue_family_chooser: qfc,
        };
    }

    if device_extensions::check_extensions(instance, device, &device::REQUIRED_DEVICE_EXTENSIONS)
        .is_err()
    {
        return PhysicalDeviceChoice {
            rating: 0,
            device,
            queue_family_chooser: qfc,
        };
    }

    if features.check_required().is_err() {
        return PhysicalDeviceChoice {
            rating: 0,
            device,
            queue_family_chooser: qfc,
        };
    }

    unsafe { instance.get_physical_device_queue_family_properties(device) }
        .into_iter()
        .enumerate()
        .for_each(|(id, prop)| qfc.inspect_queue_family(device, id.try_into().unwrap(), prop));

    if !qfc.is_complete() {
        return PhysicalDeviceChoice {
            rating: 0,
            device,
            queue_family_chooser: qfc,
        };
    }

    return PhysicalDeviceChoice {
        rating: 1,
        device,
        queue_family_chooser: qfc,
    };
}

#[derive(Debug, thiserror::Error)]
pub enum PhysicalDeviceChoiceError {
    #[error("no physical device found")]
    DeviceNotFound,
    #[error("no suitable physical device found")]
    SuitableDeviceNotFound,
}

pub struct PhysicalDeviceChoice<T: QueueFamilyChooser> {
    rating: i32,
    pub device: PhysicalDevice,
    pub queue_family_chooser: T,
}
pub fn choose_physical_device<T: QueueFamilyChooser>(
    instance: &Arc<Instance>,
    queue_family_chooser: T,
) -> Result<PhysicalDeviceChoice<T>, PhysicalDeviceChoiceError> {
    let Some(physical_device_choice) = instance
        .enumerate_physical_devices()
        .unwrap_or_else(|e| fatal_vk_error("failed to enumerate_physical_devices", e))
        .into_iter()
        .map(|device| rate_physical_device(instance, device, queue_family_chooser.clone()))
        .max_by_key(|s| s.rating)
    else {
        return Err(PhysicalDeviceChoiceError::DeviceNotFound);
    };

    if physical_device_choice.rating <= 0 {
        return Err(PhysicalDeviceChoiceError::SuitableDeviceNotFound);
    }

    Ok(physical_device_choice)
}
