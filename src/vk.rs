use ash::{
    Entry,
    vk::{self},
};
use core::fmt;
use device::{DeviceManager, swapchain::SwapchainManager};
use instance::InstanceManager;
use std::{error::Error, sync::Arc};
use surface::SurfaceManager;

use crate::window::WindowManager;

pub mod instance;
pub mod physical_device;
pub mod surface;

#[derive(Debug)]
enum VulkanInitStage {
    Entry,
    Window,
    Instance,
    Surface,
    Device,
    Swapchain,
}

impl fmt::Display for VulkanInitStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VulkanInitStage::Entry => "Entry",
                VulkanInitStage::Window => "Window",
                VulkanInitStage::Instance => "Instance",
                VulkanInitStage::Surface => "Surface",
                VulkanInitStage::Device => "Device",
                VulkanInitStage::Swapchain => "Swapchain",
            }
        )
    }
}
#[derive(Debug)]
struct VulkanInitOrderError {
    attempted_stage: VulkanInitStage,
    requiered_stage: VulkanInitStage,
}
impl fmt::Display for VulkanInitOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unable to initialize {} before {} is initalized",
            self.attempted_stage, self.requiered_stage
        )
    }
}

impl Error for VulkanInitOrderError {}

#[derive(Default)]
pub struct VulkanManager {
    entry: Option<Arc<Entry>>,
    window_manager: Option<Arc<WindowManager>>,
    instance_manager: Option<Arc<InstanceManager>>,
    surface_manager: Option<Arc<SurfaceManager>>,
    device_manager: Option<Arc<DeviceManager>>,
    swapchain_manager: Option<SwapchainManager>,
}

impl VulkanManager {
    pub fn new() -> Self {
        Self::default()
    }
    fn init_entry(&mut self) {
        self.entry = Some(Arc::new(Entry::linked()));
    }
    fn init_window_manager(&mut self) {
        self.window_manager = Some(Arc::new(WindowManager::init()));
    }
    fn init_instance(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(entry) = self.entry.clone() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Instance,
                requiered_stage: VulkanInitStage::Entry,
            }));
        };
        let Some(window_manager) = self.window_manager.as_ref() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Instance,
                requiered_stage: VulkanInitStage::Window,
            }));
        };

        let wm_required_extensions = window_manager.get_vk_extensions()?;

        let mut instance_manager = InstanceManager::init(entry)?
            .extensions(wm_required_extensions)
            .validation_layers(vec![String::from("VK_LAYER_KHRONOS_validation")])
            .application_props(String::from("WKNUP"), 1)
            .api_version(vk::make_api_version(0, 1, 1, 0));
        instance_manager.init_instance()?;

        self.instance_manager = Some(Arc::new(instance_manager));

        Ok(())
    }

    fn init_surface(&mut self) -> Result<(), Box<dyn Error>> {
        if self.instance_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Surface,
                requiered_stage: VulkanInitStage::Instance,
            }));
        };
        if self.window_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Surface,
                requiered_stage: VulkanInitStage::Window,
            }));
        };
        self.surface_manager = Some(Arc::new(SurfaceManager::init(
            self.instance_manager.clone().unwrap(),
            self.window_manager.clone().unwrap(),
        )?));
        Ok(())
    }
    fn init_device(&mut self) -> Result<(), Box<dyn Error>> {
        if self.instance_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Device,
                requiered_stage: VulkanInitStage::Instance,
            }));
        }
        if self.surface_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Device,
                requiered_stage: VulkanInitStage::Surface,
            }));
        };
        self.device_manager = Some(Arc::new(DeviceManager::init(
            self.instance_manager.clone().unwrap(),
            self.surface_manager.clone().unwrap(),
        )?));
        Ok(())
    }

    pub fn init_swapchain_manager(&mut self) -> Result<(), Box<dyn Error>> {
        if self.device_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Swapchain,
                requiered_stage: VulkanInitStage::Device,
            }));
        }
        if self.surface_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Swapchain,
                requiered_stage: VulkanInitStage::Surface,
            }));
        }

        let swapchain_manager = SwapchainManager::new(
            self.device_manager.clone().unwrap(),
            self.surface_manager.clone().unwrap(),
        );
        self.swapchain_manager = Some(swapchain_manager);
        Ok(())
    }

    pub fn create_swapchain(&mut self) -> Result<(), Box<dyn Error>> {
        if self.swapchain_manager.is_none() {
            return Err("cannot create swapchain before SwapchainManager is initialized".into());
        }
        self.swapchain_manager.as_mut().unwrap().create_swapchain(
            self.device_manager.as_ref().unwrap().get_surface_info()?,
            self.device_manager
                .as_ref()
                .unwrap()
                .get_queue_family_indices(),
        )?;
        Ok(())
    }
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let mut vulkan_manager = Self::default();
        vulkan_manager.init_entry();
        vulkan_manager.init_window_manager();
        vulkan_manager.init_instance()?;
        vulkan_manager.init_surface()?;
        vulkan_manager.init_device()?;
        vulkan_manager.init_swapchain_manager()?;
        vulkan_manager.create_swapchain()?;
        Ok(vulkan_manager)
    }
}

mod extensions;

mod validation;

mod device;
