use std::error::Error;

use ash::Entry;

#[cfg(debug_assertions)]
#[derive(Default)]
pub struct ValidationLayerManager {
    available: Option<Vec<String>>,
}

#[cfg(debug_assertions)]
impl ValidationLayerManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn enumerate(&mut self, entry: &Entry) -> Result<(), Box<dyn Error>> {
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
    pub fn check_layers(&self, layers: &[String]) -> Result<(), Box<dyn Error>> {
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
}
