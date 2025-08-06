use std::{
    error::Error,
    ffi::{CStr, CString, c_char},
    fmt,
};

use ash::{
    Entry,
    prelude::VkResult,
    vk::{self},
};

#[derive(Debug)]
pub struct ValidationLayerUnavailableError {
    layer: CString,
}

impl From<&CStr> for ValidationLayerUnavailableError {
    fn from(s: &CStr) -> Self {
        return Self {
            layer: s.to_owned(),
        };
    }
}

impl fmt::Display for ValidationLayerUnavailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "validation layer {} is not available",
            self.layer.to_str().unwrap()
        )
    }
}

impl Error for ValidationLayerUnavailableError {}

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
            available: Self::enumerate(entry).unwrap_or_else(|e| match e {
                vk::Result::ERROR_OUT_OF_HOST_MEMORY => {
                    panic!("failed to enumerate_instance_layer_properties: out of host memory")
                }
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => {
                    panic!("failed to enumerate_instance_layer_properties: out of device memory")
                }
                _ => unreachable!("all possible error cases have been covered"),
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
    pub fn check_layers(&self, layers: &[String]) -> Result<(), ValidationLayerUnavailableError> {
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

    pub fn add_layers(&mut self, layers: &[String]) -> Result<(), ValidationLayerUnavailableError> {
        self.check_layers(layers)?;
        for a_vl in self.available.iter_mut() {
            if layers.contains(&a_vl.name.to_str().unwrap().to_owned()) {
                a_vl.enabled = true;
            }
        }
        Ok(())
    }

    pub fn make_load_layer_list(&self) -> Vec<*const c_char> {
        let to_load = self
            .available
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.name.as_ptr())
            .collect();

        to_load
    }
}

#[cfg(not(debug_assertions))]
pub struct ValidationLayerManager {}

#[cfg(not(debug_assertions))]
impl ValidationLayerManager {
    pub fn init(_: &Entry) -> Self {
        Self {}
    }

    pub fn enumerate(&mut self, _: &Entry) {}
    pub fn check_layers(&self, _: &[String]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn init(entry: &Entry) -> Result<Self, Box<dyn Error>> {
        Ok(Self::new())
    }
    pub fn add_layers(&mut self, layers: &[String]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn make_load_layer_list(&mut self) -> Vec<*const c_char> {
        vec![]
    }
}
