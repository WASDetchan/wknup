use std::ffi::{CStr, CString, c_char};

use ash::{Entry, prelude::VkResult, vk};

#[derive(Debug, thiserror::Error)]
#[error("instance extension {} is not available", self.extension.to_str().unwrap())]
pub struct InstanceExtensionUnavailableError {
    extension: CString,
}

impl From<&CStr> for InstanceExtensionUnavailableError {
    fn from(s: &CStr) -> Self {
        Self {
            extension: s.to_owned(),
        }
    }
}

#[derive(Default)]
struct Extension {
    name: CString,
    enabled: bool,
}
pub struct ExtensionManager {
    available: Vec<Extension>,
}

impl ExtensionManager {
    pub fn init(entry: &Entry) -> Self {
        Self {
            available: Self::enumerate(entry).unwrap_or_else(|e| match e {
                vk::Result::ERROR_OUT_OF_HOST_MEMORY => {
                    panic!("failed to enumerate_instance_extension_properties: out of host memory")
                }
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => panic!(
                    "failed to enumerate_instance_extension_properties: out of device memory"
                ),
                _ => unreachable!("all possible error cases have been covered"),
            }),
        }
    }
    fn enumerate(entry: &Entry) -> VkResult<Vec<Extension>> {
        Ok(
            unsafe { entry.enumerate_instance_extension_properties(None)? }
                .into_iter()
                .map(|ext| Extension {
                    name: ext.extension_name_as_c_str().unwrap().into(),
                    enabled: false,
                })
                .collect(),
        )
    }
    pub fn check_extensions(
        &self,
        extensions: &[String],
    ) -> Result<(), InstanceExtensionUnavailableError> {
        for ext in extensions.iter() {
            if !self
                .available
                .iter()
                .any(|a_ext| a_ext.name.to_str().unwrap() == ext.as_str())
            {
                return Err(InstanceExtensionUnavailableError::from(
                    CString::new(ext.clone()).unwrap().as_c_str(),
                ));
            }
        }
        Ok(())
    }

    pub fn add_extensions(
        &mut self,
        extensions: &[String],
    ) -> Result<(), InstanceExtensionUnavailableError> {
        self.check_extensions(extensions)?;
        for a_ext in self.available.iter_mut() {
            if extensions.contains(&a_ext.name.to_str().unwrap().to_owned()) {
                a_ext.enabled = true;
            }
        }
        Ok(())
    }

    pub fn make_load_extension_list(&mut self) -> Vec<*const c_char> {
        self.available
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.name.as_ptr())
            .collect()
    }
}
