use std::{
    error::Error,
    ffi::{CString, c_char},
};

use ash::Entry;

struct ValidationLayer {
    name: CString,
    enabled: bool,
}

#[cfg(debug_assertions)]
#[derive(Default)]
pub struct ValidationLayerManager {
    available: Option<Vec<ValidationLayer>>,
}

#[cfg(debug_assertions)]
impl ValidationLayerManager {
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
                unsafe { entry.enumerate_instance_layer_properties()? }
                    .into_iter()
                    .map(|ext| ValidationLayer {
                        name: ext.layer_name_as_c_str().unwrap().to_owned(),
                        enabled: false,
                    })
                    .collect(),
            );
        }
        Ok(())
    }
    pub fn check_layers(&self, layers: &[String]) -> Result<(), Box<dyn Error>> {
        let Some(available) = &self.available else {
            return Err("Validation layers were not enumerated before checking.".into());
        };
        for l in layers.iter() {
            if !available
                .iter()
                .any(|vl| &vl.name.to_str().unwrap().to_owned() == l)
            {
                return Err(format!("Validation layer not found: {l}.").into());
            }
        }
        Ok(())
    }

    pub fn add_layers(&mut self, layers: &[String]) -> Result<(), Box<dyn Error>> {
        self.check_layers(layers)?;
        let available = (self.available).as_mut().unwrap().iter_mut();
        for a_vl in available {
            if layers.contains(&a_vl.name.to_str()?.to_owned()) {
                a_vl.enabled = true;
            }
        }
        Ok(())
    }

    pub fn make_load_layer_list(&self) -> Result<Vec<*const c_char>, Box<dyn Error>> {
        if self.available.is_none() {
            return Err("Validation Layers were not enumerated before loading.".into());
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

#[cfg(not(debug_assertions))]
#[derive(Default)]
pub struct ValidationLayerManager {}

#[cfg(not(debug_assertions))]
impl ValidationLayerManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn enumerate(&mut self, _: &Entry) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn check_layers(&self, _: &[String]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    pub fn init(entry: &Entry) -> Result<Self, Box<dyn Error>> {
        Ok(Self::new())
    }
    pub fn add_layers(&mut self, layers: &[String]) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn make_load_layer_list(&mut self) -> Result<Vec<*const c_char>, Box<dyn Error>> {
        Ok(vec![])
    }
}
