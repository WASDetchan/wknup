mod device_extensions;

use std::{error::Error, ffi::CStr, sync::Arc};

use ash::{
    Device,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceProperties, Queue, SurfaceKHR,
    },
};
use device_extensions::DeviceExtensionManager;

use super::{
    instance::InstanceManager,
    physical_device::{self, QueueFamilyIndices},
};

const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 1] = [c"VK_KHR_swapchain"];
struct Queues {
    graphics: Queue,
    present: Queue,
}

pub struct PhysicalDeviceInfo {
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
}

pub struct DeviceManager {
    physical_device: Option<PhysicalDevice>,
    queue_family_indices: QueueFamilyIndices,
    instance: Arc<InstanceManager>,
    device: Option<Device>,
    queues: Option<Queues>,
    surface: SurfaceKHR,
}
impl DeviceManager {
    fn new(instance: Arc<InstanceManager>, surface: SurfaceKHR) -> Self {
        Self {
            physical_device: None,
            queue_family_indices: QueueFamilyIndices::default(instance.clone(), surface),
            instance,
            device: None,
            queues: None,
            surface,
        }
    }

    fn init_physical_device(&mut self) -> Result<(), Box<dyn Error>> {
        let qfi = self.queue_family_indices.clone();
        let chosen = physical_device::choose_physical_device(&self.instance, qfi)?;
        self.queue_family_indices = chosen.queue_family_indices;
        self.physical_device = Some(chosen.device);
        Ok(())
    }

    fn init_device(&mut self) -> Result<(), Box<dyn Error>> {
        if self.physical_device.is_none() {
            return Err("cannot init device before physical_device is inited".into());
        }
        let qfi = &self.queue_family_indices;

        let graphic_present_match = qfi.graphics.unwrap() == qfi.present.unwrap();

        let graphic_info = DeviceQueueCreateInfo::default()
            .queue_family_index(qfi.graphics.unwrap() as u32)
            .queue_priorities(&[0.0f32]);
        let present_info = DeviceQueueCreateInfo::default()
            .queue_family_index(qfi.present.unwrap() as u32)
            .queue_priorities(&[0.0f32]);

        let queue_infos = if graphic_present_match {
            vec![graphic_info]
        } else {
            vec![graphic_info, present_info]
        };

        let device_features = PhysicalDeviceFeatures::default();

        let mut device_extension_manager =
            DeviceExtensionManager::init(&self.instance, self.physical_device.unwrap())?;
        device_extension_manager.add_extensions(&REQUIRED_DEVICE_EXTENSIONS)?;
        let ext_names = device_extension_manager.list_names();

        let device_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&ext_names);
        let Some(physical_device) = self.physical_device else {
            return Err("cannot init device before physical device is initialized".into());
        };
        self.device = Some(self.instance.create_device(physical_device, &device_info)?);
        Ok(())
    }

    fn init_queues(&mut self) {
        let qfi = &self.queue_family_indices;
        let device = self.device.as_ref().unwrap();

        let graphic_queue = unsafe { device.get_device_queue(qfi.graphics.unwrap() as u32, 0) };
        let present_queue = unsafe { device.get_device_queue(qfi.present.unwrap() as u32, 0) };

        let queues = Queues {
            graphics: graphic_queue,
            present: present_queue,
        };
        self.queues = Some(queues);
    }

    pub fn init(
        instance: Arc<InstanceManager>,
        surface_khr: SurfaceKHR,
    ) -> Result<Self, Box<dyn Error>> {
        let mut device_manager = Self::new(instance, surface_khr);
        device_manager.init_physical_device()?;
        device_manager.init_device()?;
        device_manager.init_queues();

        Ok(device_manager)
    }
    pub fn destroy_device(&mut self) {
        if let Some(device) = self.device.as_ref() {
            unsafe { device.destroy_device(None) };
        }
    }
}
impl Drop for DeviceManager {
    fn drop(&mut self) {
        self.destroy_device();
    }
}
