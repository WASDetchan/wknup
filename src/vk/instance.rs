use std::{
    error::Error,
    ffi::{CString, NulError},
    sync::Arc,
};

use ash::{
    Device, Entry, khr,
    vk::{
        self, ApplicationInfo, DeviceCreateInfo, ExtensionProperties, PhysicalDevice,
        QueueFamilyProperties, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
};
use sdl3::video::Window;

use super::{
    error::fatal_vk_error,
    extensions::{ExtensionManager, InstanceExtensionUnavailableError},
    physical_device::{
        PhysicalDeviceSurfaceInfo,
        features::{FeaturesInfo, PhysicalDeviceFeatures2},
    },
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

pub struct InstanceBuilder {
    extensions: Vec<String>,
    entry: Arc<Entry>,
    layers: Vec<String>,
    api_version: u32,
    apllication_props: (String, u32),
    engine_props: (String, u32),
}

impl InstanceBuilder {
    pub fn new(entry: Arc<Entry>) -> Self {
        Self {
            extensions: Vec::new(),
            layers: Vec::new(),
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

    pub fn build(self) -> Result<Instance, InstanceInitError> {
        let mut extension_manager = ExtensionManager::init(&self.entry);
        extension_manager.add_extensions(&self.extensions)?;
        let extension_names = extension_manager.make_load_extension_list();

        let mut validation_manager = ValidationLayerManager::init(&self.entry);
        validation_manager.add_layers(&self.layers)?;
        let layer_names = validation_manager.make_load_layer_list();

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
        let ash_instance = unsafe { self.entry.create_instance(&create_info, None) }
            .unwrap_or_else(|e| fatal_vk_error("failed to create_instance", e));

        Ok(Instance {
            entry: self.entry,
            instance: ash_instance,
        })
    }
}

pub struct Instance {
    instance: ash::Instance,
    entry: Arc<Entry>,
}

impl Instance {
    ///
    /// # Safety
    /// Extension instance should not be used after Instance is dropped
    ///
    pub unsafe fn make_surface_instance(&self) -> khr::surface::Instance {
        // TODO: Separate khr::surface::Instance
        khr::surface::Instance::new(&self.entry, &self.instance)
    }
    pub fn create_surface(&self, window: &Window) -> Result<SurfaceKHR, sdl3::Error> {
        window.vulkan_create_surface(self.instance.handle())
    }
    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, vk::Result> {
        unsafe { self.instance.enumerate_physical_devices() }
    }

    pub unsafe fn get_physical_device_info(&self, device: PhysicalDevice) -> PhysicalDeviceInfo {
        let mut features2 = PhysicalDeviceFeatures2::new();
        unsafe {
            features2.fill(device, &self.instance);
        }

        let features = FeaturesInfo::from_features2(features2);

        PhysicalDeviceInfo {
            properties: unsafe { self.instance.get_physical_device_properties(device) },
            features,
        }
    }
    pub unsafe fn get_physical_device_queue_family_properties(
        &self,
        physical_device: PhysicalDevice,
    ) -> Vec<QueueFamilyProperties> {
        unsafe {
            self.instance
                .get_physical_device_queue_family_properties(physical_device)
        }
    }
    pub unsafe fn get_physical_device_surface_support(
        // TODO: Separate khr::surface::Instance
        &self,
        device: PhysicalDevice,
        id: u32,
        surface: SurfaceKHR,
    ) -> Result<bool, vk::Result> {
        let s_instance = khr::surface::Instance::new(&self.entry, &self.instance);
        unsafe { s_instance.get_physical_device_surface_support(device, id, surface) }
    }

    pub unsafe fn create_device(
        &self,
        physical_device: PhysicalDevice,
        device_info: &DeviceCreateInfo,
    ) -> Result<Device, vk::Result> {
        unsafe {
            self.instance
                .create_device(physical_device, device_info, None)
        }
    }
    pub unsafe fn enumerate_device_extension_properties(
        &self,
        device: PhysicalDevice,
    ) -> Result<Vec<ExtensionProperties>, vk::Result> {
        unsafe { self.instance.enumerate_device_extension_properties(device) }
    }
    pub unsafe fn get_physical_device_surface_info(
        // TODO: Separate khr::surface::Instance
        &self,
        device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> Result<PhysicalDeviceSurfaceInfo, vk::Result> {
        unsafe {
            let s_instance = khr::surface::Instance::new(&self.entry, &self.instance);
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
        // TODO: Separate khr::swapchain::Device
        &self,
        device: &Device,
        create_info: &SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR, Box<dyn Error>> {
        let loader = khr::swapchain::Device::new(&self.instance, device);
        let swapchain = unsafe { loader.create_swapchain(create_info, None)? };
        Ok(swapchain)
    }

    ///
    /// # Safety
    /// device and swapchain should be valid
    ///
    pub unsafe fn get_swapchain_images(
        // TODO: Separate khr::swapchain::Device
        &self,
        device: &Device,
        swapchain: SwapchainKHR,
    ) -> Result<Vec<vk::Image>, Box<dyn Error>> {
        let loader = khr::swapchain::Device::new(&self.instance, device);
        let images = unsafe { loader.get_swapchain_images(swapchain)? };
        Ok(images)
    }

    ///
    /// # Safety
    /// device and swapchain should be valid
    /// swapchain will not be valid after call
    ///
    pub unsafe fn destroy_swapchain(
        // TODO: Separate khr::swapchain::Device
        &self,
        device: &Device,
        swapchain: SwapchainKHR,
    ) -> Result<(), Box<dyn Error>> {
        let loader = khr::swapchain::Device::new(&self.instance, device);
        unsafe { loader.destroy_swapchain(swapchain, None) };
        Ok(())
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
