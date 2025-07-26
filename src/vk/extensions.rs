use std::{
    error::Error,
    ffi::{CString, c_char},
};

use ash::Entry;

#[derive(Default)]
struct Extension {
    name: CString,
    enabled: bool,
}
#[derive(Default)]
pub struct ExtensionManager {
    available: Option<Vec<Extension>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(entry: &Entry) -> Result<Self, Box<dyn Error>> {
        let mut s = Self::new();
        s.enumerate(entry)?;
        Ok(s)
    }
    pub fn enumerate(&mut self, entry: &Entry) -> Result<(), Box<dyn Error>> {
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
        self.check_extensions(extensions)?;
        let available = (&mut self.available).as_mut().unwrap().iter_mut();
        for a_ext in available {
            if extensions.contains(&a_ext.name.to_str()?.to_owned()) {
                a_ext.enabled = true;
            }
        }
        Ok(())
    }

    pub fn make_load_extension_list(&mut self) -> Result<Vec<*const c_char>, Box<dyn Error>> {
        if self.available.is_none() {
            return Err("Extensions were not enumerated before loading.".into());
        };

        let to_load = self
            .available
            .as_ref()
            .unwrap()
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.name.as_ptr())
            .collect();

        Ok(to_load)
    }
}
