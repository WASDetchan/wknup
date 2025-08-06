use std::{error::Error, sync::Arc};

use ash::{khr, vk::SurfaceKHR};

use crate::window::WindowManager;

use super::instance::{self, InstanceManager};

pub struct SurfaceManager {
    instance: Arc<InstanceManager>,
    surface_instance: khr::surface::Instance,
    window: Arc<WindowManager>,
    surface: SurfaceKHR,
}

impl SurfaceManager {
    pub fn init(
        instance: Arc<InstanceManager>,
        window: Arc<WindowManager>,
    ) -> Result<Self, Box<dyn Error>> {
        let surface = window.create_surface(&instance)?;
        let surface_instance = unsafe { instance.make_surface_instance() }?;
        Ok(Self {
            instance,
            surface_instance,
            window,
            surface,
        })
    }
}

impl Drop for SurfaceManager {
    fn drop(&mut self) {
        unsafe {
            self.surface_instance.destroy_surface(self.surface, None);
        }
    }
}
