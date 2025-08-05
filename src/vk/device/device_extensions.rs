use std::{
    collections::HashSet,
    error::Error,
    ffi::{CStr, CString},
    fmt,
    os::raw::c_char,
};

use super::DeviceManager;

#[derive(Debug)]
pub struct DeviceExtensionUnavailableError {
    extension: CString,
}

impl From<&CStr> for DeviceExtensionUnavailableError {
    fn from(s: &CStr) -> Self {
        return Self {
            extension: s.to_owned(),
        };
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
    pub fn init(device: &DeviceManager) -> Result<Self, Box<dyn Error>> {
        let available = device
            .enumerate_self_device_extension_properties()?
            .into_iter()
            .map(|ext| ext.extension_name_as_c_str().unwrap().to_owned())
            .collect::<HashSet<CString>>();
        Ok(Self {
            available,
            enabled: HashSet::new(),
        })
    }
    pub fn add_extensions<T: AsRef<CStr>>(
        &mut self,
        extensions: &[T],
    ) -> Result<(), DeviceExtensionUnavailableError> {
        for ext in extensions {
            if self.available.get(ext.as_ref()) == None {
                return Err(ext.as_ref().into());
            }
            self.enabled.insert(ext.as_ref().to_owned());
        }
        Ok(())
    }
    pub fn list_names(&self) -> Vec<*const c_char> {
        self.enabled.iter().map(|ext| ext.as_ptr()).collect()
    }
}
