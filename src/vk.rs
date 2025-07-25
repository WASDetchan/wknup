use ash::{Entry, Instance, vk};
use std::{error::Error, ffi::CString, os::raw::c_char};
use validation::ValidationLayerManager;

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

        let window_manager = WindowManager::init();
        let wm_required_extensions = window_manager.get_vk_extensions();

        let mut extension_manager = ExtensionManager::new();
        extension_manager.enumerate(&entry)?;
        extension_manager.add_extensions(&wm_required_extensions?)?;
        let (ext_pp, ext_count) = extension_manager.make_load_extension_list()?;

        unsafe {
            let x = CString::from_raw(*ext_pp as *mut c_char);
            print!("{}", x.to_str()?);
        }

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: ext_count as u32,
            pp_enabled_extension_names: ext_pp,
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        let mut validation_manager = ValidationLayerManager::new();
        validation_manager.enumerate(&entry);

        Ok(Self {
            entry,
            instance,
            extension_manager,
            window_manager,
            surface: None,
        })
    }

    pub fn init_surface(&mut self) -> Result<(), Box<dyn Error>> {
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
struct Extension {
    name: CString,
    enabled: bool,
}
#[derive(Default)]
struct ExtensionManager {
    available: Option<Vec<Extension>>,
    to_load: Option<Vec<*const c_char>>,
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
                    .map(|ext| Extension {
                        name: ext.extension_name_as_c_str().unwrap().into(),
                        enabled: false,
                    })
                    .collect(),
            );
        }
        Ok(())
    }
    pub fn check_extensions(&self, extensions: &[String]) -> Result<(), Box<dyn Error>> {
        let Some(available) = &self.available else {
            return Err("Extensions were not enumerated before checking.".into());
        };
        for ext in extensions.iter() {
            if !available
                .iter()
                .any(|a_ext| a_ext.name.to_str().unwrap() == ext.as_str())
            {
                return Err(format!("Extension not found: {ext}.").into());
            }
        }
        Ok(())
    }

    pub fn add_extensions(&mut self, extensions: &[String]) -> Result<(), Box<dyn Error>> {
        self.check_extensions(extensions);
        let available = (&mut self.available).as_mut().unwrap().iter_mut();
        for a_ext in available {
            if extensions.contains(&a_ext.name.to_str()?.to_owned()) {
                a_ext.enabled = true;
            }
        }
        Ok(())
        // for ext in extensions {
        //     if !self.extensions.contains(x) {
        //         self.extensions.push(ext);
        //     }
        // }
    }

    pub fn make_load_extension_list(
        &mut self,
    ) -> Result<(*const *const c_char, usize), Box<dyn Error>> {
        let Some(available) = &self.available else {
            return Err("Extensions were not enumerated before loading.".into());
        };

        self.to_load = Some(
            available
                .iter()
                .filter(|e| e.enabled)
                .map(|e| e.name.as_ptr())
                .collect(),
        );

        Ok((
            self.to_load.as_ref().unwrap().as_ptr(),
            self.to_load.iter().count(),
        ))
    }
}

mod validation;

struct PhysicalDeviceManager {}

struct DeviceManager {}

struct QueueManager {}
