pub mod device_extensions;
pub mod swapchain;

use std::{error::Error, ffi::CStr, sync::Arc};

use ash::{
    Device,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceProperties, Queue, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};
use device_extensions::DeviceExtensionManager;

use super::{
    instance::InstanceManager,
    physical_device::{self, PhysicalDeviceSurfaceInfo, QueueFamilyIndices},
    surface::SurfaceManager,
};

pub const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 1] = [c"VK_KHR_swapchain"];
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
    surface: Arc<SurfaceManager>,
}
impl DeviceManager {
    fn new(instance: Arc<InstanceManager>, surface: Arc<SurfaceManager>) -> Self {
        Self {
            physical_device: None,
            queue_family_indices: QueueFamilyIndices::default(instance.clone(), unsafe {
                surface.raw_handle()
            }),
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
        surface: Arc<SurfaceManager>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut device_manager = Self::new(instance, surface);
        device_manager.init_physical_device()?;
        device_manager.init_device()?;
        device_manager.init_queues();

        Ok(device_manager)
    }

    pub fn create_swapchain(
        &self,
        create_info: &SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR, Box<dyn Error>> {
        let Some(device) = self.device.as_ref() else {
            return Err("cannot create swapchain before device is initialized".into());
        };
        unsafe { self.instance.create_swapchain(device, create_info) }
    }

    pub fn get_surface_info(&self) -> Result<PhysicalDeviceSurfaceInfo, Box<dyn Error>> {
        if self.physical_device.is_none() {
            return Err("cannot query surface info before physical_device is chosen".into());
        }
        physical_device::query_device_surface_info(
            &self.instance,
            self.physical_device.unwrap(),
            unsafe { self.surface.raw_handle() },
        )
    }

    pub fn get_queue_family_indices(&self) -> QueueFamilyIndices {
        self.queue_family_indices.clone()
    }

    pub unsafe fn destroy_swapchain(&self, swapchain: SwapchainKHR) -> Result<(), Box<dyn Error>> {
        unsafe {
            self.instance
                .destroy_swapchain(self.device.as_ref().unwrap(), swapchain)
        }
    }

    fn destroy_device(&mut self) {
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
