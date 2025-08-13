pub mod device_extensions;
pub mod queues;
pub mod swapchain;

use std::{error::Error, ffi::CStr, sync::Arc};

use ash::vk::{
    self, DeviceCreateInfo, DeviceQueueCreateInfo, ImageView, PhysicalDevice,
    PhysicalDeviceProperties, PipelineCache, ShaderModule, SwapchainCreateInfoKHR, SwapchainKHR,
};
use device_extensions::DeviceExtensionManager;
use queues::QueueFamilyChooser;

use super::{
    error::fatal_vk_error,
    instance::Instance,
    physical_device::{
        self, Chooser, DrawQueues, PhysicalDeviceChoice,
        features::{FeaturesInfo, PhysicalDeviceFeatures2},
    },
    surface::{PhysicalDeviceSurfaceInfo, SurfaceManager},
};

pub struct DeviceBuilder {
    queue_family_chooser: Chooser,
    instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
}

impl DeviceBuilder {
    pub fn new(instance: Arc<Instance>, surface: Arc<SurfaceManager>) -> Self {
        Self {
            queue_family_chooser: Chooser::new(instance.clone(), surface.clone()),
            instance,
            surface,
        }
    }

    pub fn build(self) -> Result<Device, Box<dyn Error>> {
        let PhysicalDeviceChoice {
            queue_family_chooser,
            device: physical_device,
            ..
        } = physical_device::choose_physical_device(
            &self.instance,
            self.queue_family_chooser.clone(),
        )?;

        let requirements = queue_family_chooser.requirements();

        let queue_infos: Vec<_> = requirements
            .iter()
            .map(|(id, priorities)| {
                DeviceQueueCreateInfo::default()
                    .queue_family_index(*id)
                    .queue_priorities(priorities)
            })
            .collect();

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

        let queues_raw = requirements
            .iter()
            .map(|(queue_family_index, priorities)| {
                (
                    *queue_family_index,
                    priorities
                        .iter()
                        .enumerate()
                        .map(|(queue_index, _)| unsafe {
                            device.get_device_queue(
                                *queue_family_index,
                                queue_index.try_into().unwrap(),
                            )
                        })
                        .collect::<Vec<vk::Queue>>(),
                )
            })
            .collect();

        let queues = queue_family_chooser.fill_queues(queues_raw);

        Ok(Device {
            instance: self.instance,
            surface: self.surface,
            physical_device,
            queue_family_chooser,
            device,
            queues,
        })
    }
}

pub const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 2] =
    [c"VK_KHR_swapchain", c"VK_KHR_vulkan_memory_model"];

pub struct PhysicalDeviceInfo {
    pub properties: PhysicalDeviceProperties,
    pub features: FeaturesInfo,
}

pub struct Device {
    instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
    physical_device: PhysicalDevice,
    queue_family_chooser: Chooser,
    device: ash::Device,
    queues: DrawQueues,
}
impl Device {
    pub fn create_swapchain(
        &self,
        create_info: &SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR, Box<dyn Error>> {
        unsafe { self.instance.create_swapchain(&self.device, create_info) }
    }

    pub fn get_surface_info(&self) -> Result<PhysicalDeviceSurfaceInfo, vk::Result> {
        self.surface
            .get_physical_device_surface_info(self.physical_device)
    }

    pub fn get_queue_family_chooser(&self) -> Chooser {
        self.queue_family_chooser.clone()
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

    pub unsafe fn create_pipeline_layout(
        &self,
        create_info: vk::PipelineLayoutCreateInfo,
    ) -> vk::PipelineLayout {
        unsafe {
            self.device
                .create_pipeline_layout(&create_info, None)
                .unwrap_or_else(|e| fatal_vk_error("failed to create pipeline layout", e))
        }
    }

    pub unsafe fn destroy_pipeline_layout(&self, layout: vk::PipelineLayout) {
        unsafe { self.device.destroy_pipeline_layout(layout, None) };
    }
    pub unsafe fn create_render_pass(
        &self,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<vk::RenderPass, vk::Result> {
        unsafe { self.device.create_render_pass(create_info, None) }
    }
    pub unsafe fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        unsafe {
            self.device.destroy_render_pass(render_pass, None);
        }
    }

    pub unsafe fn create_graphics_pipeline(
        &self,
        create_info: vk::GraphicsPipelineCreateInfo,
    ) -> Result<vk::Pipeline, vk::Result> {
        unsafe {
            let result =
                self.device
                    .create_graphics_pipelines(PipelineCache::null(), &[create_info], None);
            match result {
                Ok(ps) => Ok(ps[0]),
                Err(ps) => Err(ps.1),
            }
        }
    }

    pub unsafe fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.device.destroy_pipeline(pipeline, None);
        }
    }

    pub unsafe fn create_framebuffer(
        &self,
        create_info: &vk::FramebufferCreateInfo,
    ) -> vk::Framebuffer {
        unsafe {
            self.device
                .create_framebuffer(create_info, None)
                .unwrap_or_else(|e| fatal_vk_error("failed to create framebuffer", e))
        }
    }
    pub unsafe fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        unsafe {
            self.device.destroy_framebuffer(framebuffer, None);
        }
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.destroy_device();
    }
}
