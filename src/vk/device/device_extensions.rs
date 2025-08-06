use std::{
    collections::HashSet,
    error::Error,
    ffi::{CStr, CString},
    fmt,
    os::raw::c_char,
    sync::Arc,
};

use ash::vk::PhysicalDevice;

use crate::vk::instance::InstanceManager;

#[derive(Debug)]
pub struct DeviceExtensionUnavailableError {
    extension: CString,
}

impl From<&CStr> for DeviceExtensionUnavailableError {
    fn from(s: &CStr) -> Self {
        Self {
            extension: s.to_owned(),
        }
    }
}

impl fmt::Display for DeviceExtensionUnavailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "device extension {} is not available",
            self.extension.to_str().unwrap()
        )
    }
}

impl Error for DeviceExtensionUnavailableError {}

pub struct DeviceExtensionManager {
    available: HashSet<CString>,
    enabled: HashSet<CString>,
}

impl DeviceExtensionManager {
    pub fn init(
        instance: &Arc<InstanceManager>,
        device: PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        let available = instance
            .enumerate_device_extension_properties(device)?
            .into_iter()
            .map(|ext| ext.extension_name_as_c_str().unwrap().to_owned())
            .collect::<HashSet<CString>>();
        Ok(Self {
            available,
            enabled: HashSet::new(),
        })
    }
    pub fn check_extensions<T: AsRef<CStr>>(
        &self,
        extensions: &[T],
    ) -> Result<(), DeviceExtensionUnavailableError> {
        for ext in extensions {
            if !self.available.contains(ext.as_ref()) {
                return Err(ext.as_ref().into());
            }
        }
        Ok(())
    }
    pub fn add_extensions<T: AsRef<CStr>>(
        &mut self,
        extensions: &[T],
    ) -> Result<(), DeviceExtensionUnavailableError> {
        self.check_extensions(extensions)?;
        for ext in extensions {
            self.enabled.insert(ext.as_ref().to_owned());
        }

        Ok(())
    }
    pub fn list_names(&self) -> Vec<*const c_char> {
        self.enabled.iter().map(|ext| ext.as_ptr()).collect()
    }
}

pub fn check_extensions<T: AsRef<CStr>>(
    instance: &Arc<InstanceManager>,
    device: PhysicalDevice,
    extensions: &[T],
) -> Result<(), Box<dyn Error>> {
    let manager = DeviceExtensionManager::init(instance, device)?;
    manager.check_extensions(extensions)?;
    Ok(())
}
