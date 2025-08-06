use std::{
    error::Error,
    ffi::{CString, NulError},
    sync::Arc,
};

use ash::{
    Device, Entry, Instance, khr,
    vk::{
        self, ApplicationInfo, DeviceCreateInfo, ExtensionProperties, PhysicalDevice,
        QueueFamilyProperties, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};
use sdl3::video::Window;

use super::{
    VulkanInitStage, VulkanInitStageError,
    extensions::{ExtensionManager, InstanceExtensionUnavailableError},
    physical_device::PhysicalDeviceSurfaceInfo,
    validation::{ValidationLayerManager, ValidationLayerUnavailableError},
};
use crate::vk::device::PhysicalDeviceInfo;

#[derive(Debug, thiserror::Error)]
pub enum InstanceInitError {
    #[error("failed to init instance: {0}")]
    ExtensionUnavailable(#[from] InstanceExtensionUnavailableError),
    #[error("failed to init instance: {0}")]
    ValidatiobLayerUnavailable(#[from] ValidationLayerUnavailableError),
    #[error("failed to init instance: {0}")]
    InvalidName(#[from] NulError),
}

pub struct InstanceManager {
    instance: Option<Instance>,
    extensions: Vec<String>,
    layers: Vec<String>,
    extension_manager: ExtensionManager,
    validation_manager: ValidationLayerManager,
    entry: Arc<Entry>,
    api_version: u32,
    apllication_props: (String, u32),
    engine_props: (String, u32),
}

impl InstanceManager {
    pub fn init(entry: Arc<Entry>) -> Self {
        let extension_manager = ExtensionManager::init(&entry);
        let validation_manager = ValidationLayerManager::init(&entry);
        Self {
            instance: None,
            extensions: Vec::new(),
            layers: Vec::new(),
            extension_manager,
            validation_manager,
            entry,
            api_version: vk::make_api_version(0, 1, 0, 0),
            apllication_props: (String::new(), 0),
            engine_props: (String::new(), 0),
        }
    }
    pub fn extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }
    pub fn validation_layers(mut self, layers: Vec<String>) -> Self {
        self.layers = layers;
        self
    }

    pub fn api_version(mut self, version: u32) -> Self {
        self.api_version = version;
        self
    }
    pub fn application_props(mut self, name: String, version: u32) -> Self {
        self.apllication_props = (name, version);
        self
    }
    pub fn engine_props(mut self, name: String, version: u32) -> Self {
        self.engine_props = (name, version);
        self
    }

    pub fn init_instance(&mut self) -> Result<(), InstanceInitError> {
        self.extension_manager.add_extensions(&self.extensions)?;
        let extension_names = self.extension_manager.make_load_extension_list();

        self.validation_manager.add_layers(&self.layers)?;
        let layer_names = self.validation_manager.make_load_layer_list();

        let app_name = CString::new(self.apllication_props.0.clone())?;
        let engine_name = CString::new(self.engine_props.0.clone())?;
        let application_info = ApplicationInfo::default()
            .api_version(self.api_version)
            .application_name(&app_name)
            .application_version(self.apllication_props.1)
            .engine_name(&engine_name)
            .engine_version(self.engine_props.1);
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names);
        let instance =
            unsafe { self.entry.create_instance(&create_info, None) }.unwrap_or_else(|e| match e {
                vk::Result::ERROR_INCOMPATIBLE_DRIVER => {
                    panic!("fatal: failed to create_instance: unable to find a Vulkan driver")
                }
                vk::Result::ERROR_OUT_OF_HOST_MEMORY => {
                    panic!("fatal: failed to create_instance: a host memory allocation has failed")
                }
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => {
                    panic!(
                        "fatal: failed to create_instance: a device memory allocation has failed"
                    )
                }
                _ => unreachable!("all possible error cases have been covered"),
            });
        self.instance = Some(instance);
        Ok(())
    }

    fn require_instance(&self) -> Result<&Instance, VulkanInitStageError> {
        let Some(instance) = self.instance.as_ref() else {
            return Err(VulkanInitStageError::new(VulkanInitStage::Instance));
        };
        Ok(instance)
    }

    ///
    /// # Safety
    /// Extension instance should not be used after InstanceManager is dropped
    ///
    pub unsafe fn make_surface_instance(
        &self,
    ) -> Result<khr::surface::Instance, VulkanInitStageError> {
        let instance = self.require_instance()?;
        Ok(khr::surface::Instance::new(&self.entry, instance))
    }
    pub fn create_surface(&self, window: &Window) -> Result<SurfaceKHR, Box<dyn Error>> {
        let instance = self.require_instance()?;
        Ok(window.vulkan_create_surface(instance.handle())?)
    }
    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, Box<dyn Error>> {
        let instance = self.require_instance()?;
        Ok(unsafe { instance.enumerate_physical_devices() }?)
    }
    pub fn get_physical_device_info(
        &self,
        device: PhysicalDevice,
    ) -> Result<PhysicalDeviceInfo, VulkanInitStageError> {
        let instance = self.require_instance()?;
        Ok(PhysicalDeviceInfo {
            properties: unsafe { instance.get_physical_device_properties(device) },
            features: unsafe { instance.get_physical_device_features(device) },
        })
    }
    pub fn get_physical_device_queue_family_properties(
        &self,
        physical_device: PhysicalDevice,
    ) -> Result<Vec<QueueFamilyProperties>, VulkanInitStageError> {
        let instance = self.require_instance()?;
        Ok(unsafe { instance.get_physical_device_queue_family_properties(physical_device) })
    }
    pub fn get_physical_device_surface_support(
        &self,
        device: PhysicalDevice,
        id: u32,
        surface: SurfaceKHR,
    ) -> Result<bool, Box<dyn Error>> {
        let instance = self.require_instance()?;
        let s_instance = khr::surface::Instance::new(&self.entry, instance);
        Ok(unsafe { s_instance.get_physical_device_surface_support(device, id, surface) }?)
    }
    pub fn create_device(
        &self,
        physical_device: PhysicalDevice,
        device_info: &DeviceCreateInfo,
    ) -> Result<Device, Box<dyn Error>> {
        let instance = self.require_instance()?;
        Ok(unsafe { instance.create_device(physical_device, device_info, None) }?)
    }
    pub fn enumerate_device_extension_properties(
        &self,
        device: PhysicalDevice,
    ) -> Result<Vec<ExtensionProperties>, Box<dyn Error>> {
        let instance = self.require_instance()?;
        Ok(unsafe { instance.enumerate_device_extension_properties(device)? })
    }
    pub fn get_physical_device_surface_info(
        &self,
        device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> Result<PhysicalDeviceSurfaceInfo, Box<dyn Error>> {
        unsafe {
            let instance = self.require_instance()?;
            let s_instance = khr::surface::Instance::new(&self.entry, instance);
            let capabilities =
                s_instance.get_physical_device_surface_capabilities(device, surface)?;
            let formats = s_instance.get_physical_device_surface_formats(device, surface)?;
            let present_modes =
                s_instance.get_physical_device_surface_present_modes(device, surface)?;
            Ok(PhysicalDeviceSurfaceInfo {
                capabilities,
                formats,
                present_modes,
            })
        }
    }

    ///
    /// # Safety
    /// device should be valid
    ///
    pub unsafe fn create_swapchain(
        &self,
        device: &Device,
        create_info: &SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR, Box<dyn Error>> {
        let instance = self.require_instance()?;
        let loader = khr::swapchain::Device::new(instance, device);
        let swapchain = unsafe { loader.create_swapchain(create_info, None)? };
        Ok(swapchain)
    }

    pub unsafe fn get_swapchain_images(
        &self,
        device: &Device,
        swapchain: SwapchainKHR,
    ) -> Result<Vec<vk::Image>, Box<dyn Error>> {
        let instance = self.require_instance()?;
        let loader = khr::swapchain::Device::new(instance, device);
        let images = unsafe { loader.get_swapchain_images(swapchain)? };
        Ok(images)
    }
    ///
    /// # Safety
    /// device and swapchain should be valid
    /// swapchain will not be valid after call
    ///
    pub unsafe fn destroy_swapchain(
        &self,
        device: &Device,
        swapchain: SwapchainKHR,
    ) -> Result<(), Box<dyn Error>> {
        let instance = self.require_instance()?;
        let loader = khr::swapchain::Device::new(instance, device);
        unsafe { loader.destroy_swapchain(swapchain, None) };
        Ok(())
    }
}

impl Drop for InstanceManager {
    fn drop(&mut self) {
        if let Some(instance) = self.instance.as_ref() {
            unsafe {
                instance.destroy_instance(None);
            }
        }
    }
}
