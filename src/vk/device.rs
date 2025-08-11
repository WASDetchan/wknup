pub mod device_extensions;
pub mod swapchain;

use std::{error::Error, ffi::CStr, sync::Arc};

use ash::vk::{
    self, DeviceCreateInfo, DeviceQueueCreateInfo, ImageView, PhysicalDevice,
    PhysicalDeviceProperties, Queue, ShaderModule, SwapchainCreateInfoKHR, SwapchainKHR,
};
use device_extensions::DeviceExtensionManager;

use super::{
    error::fatal_vk_error,
    instance::{Instance, surface::SurfaceInstance},
    physical_device::{
        self, PhysicalDeviceChoice, PhysicalDeviceSurfaceInfo, QueueFamilyIndices,
        features::{FeaturesInfo, PhysicalDeviceFeatures2},
    },
    surface::SurfaceManager,
};

pub struct DeviceBuilder {
    queue_family_indices: QueueFamilyIndices,
    instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
}

impl DeviceBuilder {
    pub fn new(instance: Arc<Instance>, surface: Arc<SurfaceManager>) -> Self {
        Self {
            queue_family_indices: QueueFamilyIndices::default(instance.clone(), unsafe {
                surface.raw_handle()
            }),
            instance,
            surface,
        }
    }

    pub fn build(self) -> Result<Device, Box<dyn Error>> {
        let PhysicalDeviceChoice {
            queue_family_indices,
            device: physical_device,
        } = physical_device::choose_physical_device(
            &self.instance,
            self.queue_family_indices.clone(),
        )?;

        let graphic_present_match =
            queue_family_indices.graphics.unwrap() == queue_family_indices.present.unwrap();

        let graphic_info = DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_indices.graphics.unwrap() as u32)
            .queue_priorities(&[0.0f32]);
        let present_info = DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_indices.present.unwrap() as u32)
            .queue_priorities(&[0.0f32]);

        let queue_infos = if graphic_present_match {
            vec![graphic_info]
        } else {
            vec![graphic_info, present_info]
        };

        let features2 = PhysicalDeviceFeatures2::new_required();

        let device_features = features2.features();
        let mut next = features2.next();

        let mut device_extension_manager =
            DeviceExtensionManager::init(&self.instance, physical_device)?;
        device_extension_manager.add_extensions(&REQUIRED_DEVICE_EXTENSIONS)?;
        let ext_names = device_extension_manager.list_names();

        let device_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&ext_names)
            .push_next(&mut next);

        let device = unsafe { self.instance.create_device(physical_device, &device_info) }?;

        let graphic_queue =
            unsafe { device.get_device_queue(queue_family_indices.graphics.unwrap() as u32, 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family_indices.present.unwrap() as u32, 0) };

        let queues = Queues {
            graphics: graphic_queue,
            present: present_queue,
        };

        Ok(Device {
            instance: self.instance,
            surface: self.surface,
            physical_device,
            queue_family_indices,
            device,
            queues,
        })
    }
}

pub const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 2] =
    [c"VK_KHR_swapchain", c"VK_KHR_vulkan_memory_model"];

#[allow(dead_code)]
struct Queues {
    graphics: Queue,
    present: Queue,
}

pub struct PhysicalDeviceInfo {
    pub properties: PhysicalDeviceProperties,
    pub features: FeaturesInfo,
}

pub struct Device {
    instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
    physical_device: PhysicalDevice,
    queue_family_indices: QueueFamilyIndices,
    device: ash::Device,
    queues: Queues,
}
impl Device {
    pub fn create_swapchain(
        &self,
        create_info: &SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR, Box<dyn Error>> {
        unsafe { self.instance.create_swapchain(&self.device, create_info) }
    }

    pub fn get_surface_info(&self) -> Result<PhysicalDeviceSurfaceInfo, vk::Result> {
        let surface_instance = Arc::new(SurfaceInstance::new(self.instance.clone()));
        unsafe {
            physical_device::query_device_surface_info(
                surface_instance,
                self.physical_device,
                self.surface.raw_handle(),
            )
        }
    }

    pub fn get_queue_family_indices(&self) -> QueueFamilyIndices {
        self.queue_family_indices.clone()
    }

    pub unsafe fn destroy_swapchain(&self, swapchain: SwapchainKHR) -> Result<(), Box<dyn Error>> {
        unsafe { self.instance.destroy_swapchain(&self.device, swapchain) }
    }

    fn destroy_device(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }

    pub unsafe fn get_swapchain_images(
        &self,
        swapchain: SwapchainKHR,
    ) -> Result<Vec<vk::Image>, Box<dyn Error>> {
        unsafe { self.instance.get_swapchain_images(&self.device, swapchain) }
    }

    pub unsafe fn create_image_view(&self, create_info: &vk::ImageViewCreateInfo) -> vk::ImageView {
        unsafe {
            self.device
                .create_image_view(create_info, None)
                .unwrap_or_else(|e| fatal_vk_error("failed to create_image_view", e))
        }
    }

    pub unsafe fn destroy_image_view(&self, view: ImageView) {
        unsafe {
            self.device.destroy_image_view(view, None);
        }
    }

    pub unsafe fn create_shader_module(&self, shader: &[u32]) -> ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::default().code(shader);
        unsafe {
            self.device
                .create_shader_module(&create_info, None)
                .unwrap_or_else(|e| fatal_vk_error("failed to create_shader_module", e))
        }
    }

    pub unsafe fn destroy_shader_module(&self, shader: vk::ShaderModule) {
        unsafe {
            self.device.destroy_shader_module(shader, None);
        }
    }

    pub fn create_pipeline_layout(
        &self,
        create_info: vk::PipelineLayoutCreateInfo,
    ) -> vk::PipelineLayout {
        unsafe {
            self.device
                .create_pipeline_layout(&create_info, None)
                .unwrap_or_else(|e| fatal_vk_error("failed to create pipeline layout", e))
        }
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.destroy_device();
    }
}
