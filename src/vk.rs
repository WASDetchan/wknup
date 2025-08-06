use ash::{
    Entry,
    vk::{self},
};
use device::{DeviceManager, swapchain::SwapchainManager};
use instance::InstanceManager;
use std::{error::Error, sync::Arc};
use surface::SurfaceManager;

use crate::window::WindowManager;

pub mod instance;
pub mod physical_device;
pub mod surface;

#[derive(Debug, strum::Display)]
enum VulkanInitStage {
    Entry,
    Instance,
    Surface,
    Device,
    Swapchain,
}

#[derive(Debug, thiserror::Error)]
#[error("{requiered_stage} must be initialized")]
struct VulkanInitStageError {
    requiered_stage: VulkanInitStage,
}

impl VulkanInitStageError {
    fn new(stage: VulkanInitStage) -> Self {
        Self {
            requiered_stage: stage,
        }
    }
}

#[derive(Default)]
pub struct VulkanManager {
    entry: Option<Arc<Entry>>,
    instance_manager: Option<Arc<InstanceManager>>,
    surface_manager: Option<Arc<SurfaceManager>>,
    device_manager: Option<Arc<DeviceManager>>,
    swapchain_manager: Option<SwapchainManager>,
}

impl VulkanManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn require_init_stage(&self, stage: VulkanInitStage) -> Result<(), VulkanInitStageError> {
        if match stage {
            VulkanInitStage::Entry => self.entry.is_none(),
            VulkanInitStage::Instance => self.instance_manager.is_none(),
            VulkanInitStage::Surface => self.surface_manager.is_none(),
            VulkanInitStage::Device => self.device_manager.is_none(),
            VulkanInitStage::Swapchain => self.swapchain_manager.is_none(),
        } {
            Err(VulkanInitStageError::new(stage))
        } else {
            Ok(())
        }
    }

    fn init_entry(&mut self) {
        self.entry = Some(Arc::new(Entry::linked()));
    }

    fn init_instance(&mut self, window: &WindowManager) -> Result<(), Box<dyn Error>> {
        self.require_init_stage(VulkanInitStage::Entry)?;

        let wm_required_extensions = window.get_vk_extensions()?;

        let mut instance_manager = InstanceManager::init(self.entry.clone().unwrap())
            .extensions(wm_required_extensions)
            .validation_layers(vec![String::from("VK_LAYER_KHRONOS_validation")])
            .application_props(String::from("WKNUP"), 1)
            .api_version(vk::make_api_version(0, 1, 1, 0));
        instance_manager.init_instance()?;

        self.instance_manager = Some(Arc::new(instance_manager));

        Ok(())
    }

    fn init_surface(&mut self, window: &WindowManager) -> Result<(), Box<dyn Error>> {
        self.require_init_stage(VulkanInitStage::Instance)?;
        self.surface_manager = Some(Arc::new(SurfaceManager::init(
            self.instance_manager.clone().unwrap(),
            window,
        )?));
        Ok(())
    }
    fn init_device(&mut self) -> Result<(), Box<dyn Error>> {
        self.require_init_stage(VulkanInitStage::Instance)?;
        self.require_init_stage(VulkanInitStage::Surface)?;
        self.device_manager = Some(Arc::new(DeviceManager::init(
            self.instance_manager.clone().unwrap(),
            self.surface_manager.clone().unwrap(),
        )?));
        Ok(())
    }

    pub fn init_swapchain_manager(&mut self) -> Result<(), Box<dyn Error>> {
        self.require_init_stage(VulkanInitStage::Device)?;
        self.require_init_stage(VulkanInitStage::Device)?;

        let swapchain_manager = SwapchainManager::new(
            self.device_manager.clone().unwrap(),
            self.surface_manager.clone().unwrap(),
        );
        self.swapchain_manager = Some(swapchain_manager);
        self.create_swapchain()?;
        Ok(())
    }

    pub fn create_swapchain(&mut self) -> Result<(), Box<dyn Error>> {
        self.require_init_stage(VulkanInitStage::Swapchain)?;
        self.swapchain_manager.as_mut().unwrap().create_swapchain(
            self.device_manager.as_ref().unwrap().get_surface_info()?,
            self.device_manager
                .as_ref()
                .unwrap()
                .get_queue_family_indices(),
        )?;
        Ok(())
    }
    pub fn init(window: &WindowManager) -> Result<Self, Box<dyn Error>> {
        let mut vulkan_manager = Self::default();
        vulkan_manager.init_entry();
        vulkan_manager.init_instance(window)?;
        vulkan_manager.init_surface(window)?;
        vulkan_manager.init_device()?;
        vulkan_manager.init_swapchain_manager()?;
        Ok(vulkan_manager)
    }
}

mod extensions;

mod validation;

mod device;
