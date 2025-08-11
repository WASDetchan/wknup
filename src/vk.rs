use ash::{
    Entry,
    vk::{self},
};
use device::{Device, DeviceBuilder, swapchain::SwapchainManager};
use instance::{Instance, InstanceBuilder};
use std::{error::Error, sync::Arc};
use surface::SurfaceManager;

use crate::window::WindowManager;

pub mod error;
pub mod instance;
mod physical_device;
pub mod pipeline;
pub mod shader;
mod surface;

pub struct VulkanBuilder<'a> {
    window: &'a WindowManager,
}

impl<'a> VulkanBuilder<'a> {
    pub fn new(window: &'a WindowManager) -> Self {
        VulkanBuilder { window }
    }

    fn init_entry() -> Arc<Entry> {
        Arc::new(Entry::linked())
    }

    fn init_instance(
        window: &WindowManager,
        entry: Arc<Entry>,
    ) -> Result<Arc<Instance>, Box<dyn Error>> {
        let wm_required_extensions = window.get_vk_extensions()?;

        let instance = InstanceBuilder::new(entry)
            .extensions(wm_required_extensions)
            .validation_layers(vec![String::from("VK_LAYER_KHRONOS_validation")])
            .application_props(String::from("WKNUP"), 1)
            .api_version(vk::make_api_version(0, 1, 1, 0))
            .build()?;
        Ok(Arc::new(instance))
    }
    fn init_surface(
        window: &WindowManager,
        instance: Arc<Instance>,
    ) -> Result<Arc<SurfaceManager>, sdl3::Error> {
        Ok(Arc::new(SurfaceManager::init(instance, window)?))
    }
    fn init_device(
        instance: Arc<Instance>,
        surface: Arc<SurfaceManager>,
    ) -> Result<Arc<Device>, Box<dyn Error>> {
        Ok(Arc::new(DeviceBuilder::new(instance, surface).build()?))
    }
    fn init_swapchain_manager(
        surface: Arc<SurfaceManager>,
        device: Arc<Device>,
    ) -> Arc<SwapchainManager> {
        Arc::new(SwapchainManager::new(device, surface))
    }
    pub fn build(self) -> Result<Vulkan, Box<dyn Error>> {
        let entry = Self::init_entry();
        let instance = Self::init_instance(self.window, Arc::clone(&entry))?;
        let surface = Self::init_surface(self.window, Arc::clone(&instance))?;
        let device = Self::init_device(Arc::clone(&instance), Arc::clone(&surface))?;
        let swapchain_manager =
            Self::init_swapchain_manager(Arc::clone(&surface), Arc::clone(&device));
        Ok(Vulkan {
            entry,
            instance,
            surface,
            device,
            swapchain_manager,
        })
    }
}
pub struct Vulkan {
    entry: Arc<Entry>,
    instance: Arc<Instance>,
    surface: Arc<SurfaceManager>,
    device: Arc<Device>,
    swapchain_manager: Arc<SwapchainManager>,
}

impl Vulkan {
    pub fn get_entry(&self) -> Arc<Entry> {
        Arc::clone(&self.entry)
    }
    pub fn get_instance(&self) -> Arc<Instance> {
        Arc::clone(&self.instance)
    }
    pub fn get_surface(&self) -> Arc<SurfaceManager> {
        Arc::clone(&self.surface)
    }
    pub fn get_device(&self) -> Arc<Device> {
        Arc::clone(&self.device)
    }
    pub fn get_swapchain_manager(&self) -> Arc<SwapchainManager> {
        Arc::clone(&self.swapchain_manager)
    }
}

mod extensions;

mod validation;

mod device;
