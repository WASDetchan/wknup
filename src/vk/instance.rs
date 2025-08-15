pub mod surface;

use std::{
    error::Error,
    ffi::{CString, NulError},
    fmt::{self},
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
    physical_device::features::{FeaturesInfo, PhysicalDeviceFeatures2},
    validation::{ValidationLayerManager, ValidationLayerUnavailableError},
};
use crate::vk::{device::PhysicalDeviceInfo, validation};

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

    pub fn build(mut self) -> Result<Instance, InstanceInitError> {
        if cfg!(debug_assertions) {
            self.extensions.push(String::from("VK_EXT_debug_utils"));
        }

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

        let debug_messenger = if cfg!(debug_assertions) {
            let loader = ash::ext::debug_utils::Instance::new(&self.entry, &ash_instance);
            Some(unsafe { validation::create_debug_messenger(loader) })
        } else {
            None
        };

        let instance = Instance {
            entry: self.entry,
            instance: ash_instance,
            debug_messenger,
        };

        log::info!("Created {:?}", instance);
        log::debug!(
            instance:?;
            "
{:?} Info:
api_version: {};
app: name: {}, version: {};
engine: name: {}, version: {};
extension: {:?};
validation layers: {:?};",
            instance,
            self.api_version,
            self.apllication_props.0,
            self.apllication_props.1,
            self.engine_props.0,
            self.engine_props.1,
            self.extensions,
            if cfg!(debug_assertions) {
                self.layers
            } else {
                Vec::new()
            },
        );

        Ok(instance)
    }
}

pub struct Instance {
    instance: ash::Instance,
    entry: Arc<Entry>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Instance {
    ///
    /// # Safety
    /// khr::surface::Instance should not be used after parent instance is destroyed
    ///
    unsafe fn make_surface_instance(&self) -> khr::surface::Instance {
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

    pub(in crate::vk) unsafe fn raw_handle(&self) -> ash::Instance {
        self.instance.clone()
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instance {:?}", self.instance.handle())
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let Some(dm) = self.debug_messenger {
                let loader = ash::ext::debug_utils::Instance::new(&self.entry, &self.instance);
                loader.destroy_debug_utils_messenger(dm, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
