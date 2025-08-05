use ash::{
    Entry,
    vk::{self},
};
use core::fmt;
use device::DeviceManager;
use instance::InstanceManager;
use std::{error::Error, sync::Arc};

use crate::window::WindowManager;

pub mod instance;
pub mod physical_device;

#[derive(Debug)]
enum VulkanInitStage {
    Entry,
    WindowManager,
    InstanceManager,
    Surface,
    Device,
}

impl fmt::Display for VulkanInitStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VulkanInitStage::Entry => "Entry",
                VulkanInitStage::WindowManager => "WindowManager",
                VulkanInitStage::InstanceManager => "InstanceManager",
                VulkanInitStage::Surface => "Surface",
                VulkanInitStage::Device => "Device",
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
    instance_manager: Option<Arc<InstanceManager>>,
    window_manager: Option<WindowManager>,
    device_manager: Option<DeviceManager>,
}

impl VulkanManager {
    pub fn new() -> Self {
        Self::default()
    }
    fn init_entry(&mut self) {
        self.entry = Some(Arc::new(Entry::linked()));
    }
    fn init_window_manager(&mut self) {
        self.window_manager = Some(WindowManager::init());
    }
    fn init_instance(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(entry) = self.entry.clone() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::InstanceManager,
                requiered_stage: VulkanInitStage::Entry,
            }));
        };
        let Some(window_manager) = self.window_manager.as_ref() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::InstanceManager,
                requiered_stage: VulkanInitStage::WindowManager,
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
                requiered_stage: VulkanInitStage::InstanceManager,
            }));
        };
        self.window_manager
            .as_mut()
            .expect("window_manager is always initialized at this point")
            .init_surface(Arc::clone(self.instance_manager.as_ref().unwrap()))?;
        Ok(())
    }
    fn init_device(&mut self) -> Result<(), Box<dyn Error>> {
        if self.instance_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Device,
                requiered_stage: VulkanInitStage::InstanceManager,
            }));
        }
        let Some(surface) = self.window_manager.as_ref().unwrap().surface() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Device,
                requiered_stage: VulkanInitStage::Surface,
            }));
        };
        self.device_manager = Some(DeviceManager::init(
            self.instance_manager.clone().unwrap(),
            surface,
        )?);
        Ok(())
    }
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let mut vulkan_manager = Self::default();
        vulkan_manager.init_entry();
        vulkan_manager.init_window_manager();
        vulkan_manager.init_instance()?;
        vulkan_manager.init_surface()?;
        vulkan_manager.init_device()?;
        Ok(vulkan_manager)
    }
}

mod extensions;

mod validation;

mod device;
