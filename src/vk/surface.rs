use std::sync::Arc;

use ash::{khr, vk::SurfaceKHR};

use crate::window::WindowManager;

use super::instance::Instance;

pub struct SurfaceManager {
    _instance: Arc<Instance>,
    surface_instance: khr::surface::Instance,
    surface: SurfaceKHR,
}

impl SurfaceManager {
    pub fn init(instance: Arc<Instance>, window: &WindowManager) -> Result<Self, sdl3::Error> {
        let surface = window.create_surface(&instance)?;
        let surface_instance = unsafe { instance.make_surface_instance() };
        Ok(Self {
            _instance: instance,
            surface_instance,
            surface,
        })
    }

    ///
    /// # Safety
    /// SurfaceKHR should not be destroyed via raw handle
    ///
    pub unsafe fn raw_handle(&self) -> SurfaceKHR {
        self.surface
    }
}

impl Drop for SurfaceManager {
    fn drop(&mut self) {
        unsafe {
            self.surface_instance.destroy_surface(self.surface, None);
        }
    }
}
