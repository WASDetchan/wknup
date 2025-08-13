pub mod device_extensions;
pub mod queues;

use std::{error::Error, ffi::CStr, sync::Arc};

use ash::vk::{
    self, DeviceCreateInfo, DeviceQueueCreateInfo, ImageView, PhysicalDeviceProperties,
    PipelineCache, ShaderModule, SwapchainCreateInfoKHR, SwapchainKHR,
};
use device_extensions::DeviceExtensionManager;
use queues::{Queue, QueueFamilySelector, Queues};

use super::{
    error::fatal_vk_error,
    instance::Instance,
    physical_device::{
        self,
        features::{FeaturesInfo, PhysicalDeviceFeatures2},
    },
    surface::{PhysicalDeviceSurfaceInfo, SurfaceManager},
};

pub struct DeviceBuilder<S: QueueFamilySelector> {
    queue_family_selector: S,
    instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
}

impl<S: QueueFamilySelector> DeviceBuilder<S> {
    pub fn new(
        instance: Arc<Instance>,
        surface: Arc<SurfaceManager>,
        queue_family_selector: S,
    ) -> Self {
        Self {
            queue_family_selector,
            instance,
            surface,
        }
    }

    pub fn build(self) -> Result<(Device, S), Box<dyn Error>> {
        let physical_device_choice = physical_device::choose_physical_device(
            &self.instance,
            self.queue_family_selector.clone(),
        )?;

        let physical_device = physical_device_choice.device;
        let queue_family_selector = physical_device_choice.queue_family_selector.clone();

        let requirements = queue_family_selector.requirements();

        let len = physical_device_choice.queue_counts.len();
        let mut queue_counts = Vec::new();
        queue_counts.resize(len, 0);
        for (id, priorities) in requirements.iter() {
            if *id as usize >= len
                || queue_counts[*id as usize] != 0
                || (physical_device_choice.queue_counts[*id as usize] as usize) < priorities.len()
            {
                panic!("queue selector returned invalid requirements!");
            }
            queue_counts[*id as usize] = priorities.len();
        }

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

        Ok((
            Device {
                instance: self.instance,
                surface: self.surface,
                physical_device: physical_device_choice.device,
                device,
                queue_counts,
            },
            physical_device_choice.queue_family_selector,
        ))
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
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue_counts: Vec<usize>,
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

    pub fn get_queue_family_count(&self) -> usize {
        self.queue_counts.len()
    }

    pub fn get_queue_counts(&self) -> Vec<usize> {
        self.queue_counts.clone()
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

    pub(in crate::vk) unsafe fn raw_handle(&self) -> ash::Device {
        self.device.clone()
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        self.destroy_device();
    }
}
pub fn fill_selector(device: Arc<Device>, selector: impl QueueFamilySelector) -> impl Queues {
    let requirements = selector.requirements();

    let len = device.get_queue_family_count();
    let queue_counts = device.get_queue_counts();
    for (id, priorities) in requirements.iter() {
        if *id as usize >= len || (queue_counts[*id as usize] as usize) < priorities.len() {
            panic!("queue selector returned invalid requirements!");
        }
    }
    let queues_raw = requirements
        .iter()
        .map(|(queue_family_index, priorities)| {
            (
                *queue_family_index,
                priorities
                    .iter()
                    .enumerate()
                    .map(|(queue_index, _)| unsafe {
                        Queue::new(
                            Arc::clone(&device),
                            device.raw_handle().get_device_queue(
                                *queue_family_index,
                                queue_index.try_into().unwrap(),
                            ),
                        )
                    })
                    .collect::<Vec<Queue>>(),
            )
        })
        .collect();

    let queues = selector.fill_queues(queues_raw);

    queues
}
