use std::ffi::{self, CStr, CString, c_char};

use ash::{Entry, prelude::VkResult, vk};

use super::error::fatal_vk_error;

#[derive(Debug, thiserror::Error)]
#[error("validation layer {} is not available", self.layer.to_str().unwrap())]
pub struct ValidationLayerUnavailableError {
    layer: CString,
}

impl From<&CStr> for ValidationLayerUnavailableError {
    fn from(s: &CStr) -> Self {
        Self {
            layer: s.to_owned(),
        }
    }
}

struct ValidationLayer {
    name: CString,
    enabled: bool,
}

#[cfg(debug_assertions)]
pub struct ValidationLayerManager {
    available: Vec<ValidationLayer>,
}

#[cfg(debug_assertions)]
impl ValidationLayerManager {
    pub fn init(entry: &Entry) -> Self {
        Self {
            available: Self::enumerate(entry).unwrap_or_else(|e| {
                fatal_vk_error(
                    "failed to enumerate_instance_layer_properties",
                    e,
                )
            }),
        }
    }
    fn enumerate(entry: &Entry) -> VkResult<Vec<ValidationLayer>> {
        Ok(unsafe { entry.enumerate_instance_layer_properties() }?
            .into_iter()
            .map(|ext| ValidationLayer {
                name: ext.layer_name_as_c_str().unwrap().to_owned(),
                enabled: false,
            })
            .collect())
    }
    pub fn check_layers(
        &self,
        layers: &[String],
    ) -> Result<(), ValidationLayerUnavailableError> {
        for l in layers.iter() {
            if !self
                .available
                .iter()
                .any(|vl| &vl.name.to_str().unwrap().to_owned() == l)
            {
                return Err(ValidationLayerUnavailableError::from(
                    CString::new(l.clone()).unwrap().as_c_str(),
                ));
            }
        }
        Ok(())
    }

    pub fn add_layers(
        &mut self,
        layers: &[String],
    ) -> Result<(), ValidationLayerUnavailableError> {
        self.check_layers(layers)?;
        for a_vl in self.available.iter_mut() {
            if layers.contains(&a_vl.name.to_str().unwrap().to_owned()) {
                a_vl.enabled = true;
            }
        }
        Ok(())
    }

    pub fn make_load_layer_list(&self) -> Vec<*const c_char> {
        self.available
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.name.as_ptr())
            .collect()
    }
}

#[cfg(not(debug_assertions))]
pub struct ValidationLayerManager {}

#[cfg(not(debug_assertions))]
impl ValidationLayerManager {
    pub fn init(_: &Entry) -> Self {
        Self {}
    }

    pub fn enumerate(_: &Entry) -> VkResult<Vec<ValidationLayer>> {
        Ok(Vec::new())
    }
    pub fn check_layers(
        &self,
        _: &[String],
    ) -> Result<(), ValidationLayerUnavailableError> {
        Ok(())
    }
    pub fn add_layers(
        &mut self,
        layers: &[String],
    ) -> Result<(), ValidationLayerUnavailableError> {
        Ok(())
    }

    pub fn make_load_layer_list(&mut self) -> Vec<*const c_char> {
        vec![]
    }
}

unsafe extern "system" fn log_validation(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _p_user_data: *mut ffi::c_void,
) -> u32 {
    use log::Level;
    use vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
    let level = match message_severity {
        Severity::VERBOSE => Level::Debug,
        Severity::INFO => Level::Info,
        Severity::WARNING => Level::Warn,
        Severity::ERROR => Level::Error,
        _ => unreachable!("All severtiry levels were checked"),
    };
    log::log!(level, "{}", unsafe {
        CStr::from_ptr((*p_callback_data).p_message)
            .to_str()
            .unwrap()
    });
    0
}

pub(in crate::vk) unsafe fn create_debug_messenger(
    loader: ash::ext::debug_utils::Instance,
) -> vk::DebugUtilsMessengerEXT {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Severity;
    use vk::DebugUtilsMessageTypeFlagsEXT as Type;
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            Severity::VERBOSE
                | Severity::INFO
                | Severity::WARNING
                | Severity::ERROR,
        )
        .message_type(Type::GENERAL | Type::PERFORMANCE | Type::VALIDATION)
        .pfn_user_callback(Some(log_validation));
    unsafe {
        loader
            .create_debug_utils_messenger(&create_info, None)
            .unwrap_or_else(|error| {
                fatal_vk_error("create_debug_utils_messenger", error)
            })
    }
}
