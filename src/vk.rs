use ash::{Entry, Instance, vk};
use std::error::Error;

use crate::window::WindowManager;

pub struct VulkanManager {
    entry: Entry,
    instance: Instance,
    extension_manager: ExtensionManager,
    window_manager: WindowManager,
    surface: Option<vk::SurfaceKHR>,
}

impl VulkanManager {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let entry = Entry::linked();
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 1, 0),
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        let mut extension_manager = ExtensionManager::new();
        extension_manager.enumerate(&entry)?;

        let window_manager = WindowManager::init();

        Ok(Self {
            entry,
            instance,
            extension_manager,
            window_manager,
            surface: None,
        })
    }

    pub fn init_surface(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_extensions(self.window_manager.get_vk_extensions()?)?;
        self.surface = Some(
            self.window_manager
                .create_vk_surface(self.instance.handle())?,
        );
        Ok(())
    }

    pub fn check_extensions(&self, extensions: Vec<String>) -> Result<(), Box<dyn Error>> {
        self.extension_manager.check_extensions(&extensions)
    }
}
impl Drop for VulkanManager {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Default)]
struct ExtensionManager {
    available: Option<Vec<String>>,
}

impl ExtensionManager {
    fn new() -> Self {
        Self::default()
    }
    fn enumerate(&mut self, entry: &Entry) -> Result<(), Box<dyn Error>> {
        if self.available.is_none() {
            self.available = Some(
                unsafe { entry.enumerate_instance_extension_properties(None)? }
                    .into_iter()
                    .map(|ext| {
                        ext.extension_name_as_c_str()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect(),
            );
        }
        Ok(())
    }
    pub fn check_extensions(&self, extensions: &Vec<String>) -> Result<(), Box<dyn Error>> {
        let Some(available) = &self.available else {
            return Err("Extensions were not enumerated before checking.".into());
        };
        for ext in extensions.iter() {
            if !available.contains(ext) {
                return Err(format!(
                    "Extension not found: {ext}. Available_extensions: {:?}.",
                    available
                )
                .into());
            }
        }
        Ok(())
    }
}

#[cfg(debug_assertions)]
#[derive(Default)]
struct ValidationLayerManager {
    available: Option<Vec<String>>,
}

#[cfg(debug_assertions)]
impl ValidationLayerManager {
    fn new() -> Self {
        Self::default()
    }
    fn enumerate(&mut self, entry: &Entry) -> Result<(), Box<dyn Error>> {
        if self.available.is_none() {
            self.available = Some(
                unsafe { entry.enumerate_instance_layer_properties()? }
                    .into_iter()
                    .map(|ext| {
                        ext.layer_name_as_c_str()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                    })
                    .collect(),
            );
        }
        Ok(())
    }
    pub fn check_layers(&self, layers: &Vec<String>) -> Result<(), Box<dyn Error>> {
        let Some(available) = &self.available else {
            return Err("Validation layers were not enumerated before checking.".into());
        };
        for l in layers.iter() {
            if !available.contains(l) {
                return Err(format!(
                    "Validation layer not found: {l}. Available: {:?}.",
                    available
                )
                .into());
            }
        }
        Ok(())
    }
}

#[cfg(not(debug_assertions))]
#[derive(Default)]
struct ValidationLayerManager {}

#[cfg(not(debug_assertions))]
impl ValidationLayerManager {
    fn new() -> Self {
        Self::default()
    }
    fn enumerate(&mut self, entry: &Entry) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn check_layers(&self, layers: &Vec<String>) -> Result<(), Box<dyn Error>> {
        let Some(available) = &self.available else {
            return Err("Validation layers were not enumerated before checking.".into());
        };
        for l in layers.iter() {
            if !available.contains(l) {
                return Err(format!(
                    "Validation layer not found: {l}. Available: {:?}.",
                    available
                )
                .into());
            }
        }
        Ok(())
    }
}

struct PhysicalDeviceManager {}

struct DeviceManager {}

struct QueueManager {}
