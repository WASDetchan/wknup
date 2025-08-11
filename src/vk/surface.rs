use std::sync::Arc;

use ash::vk::SurfaceKHR;

use crate::window::WindowManager;

use super::instance::{Instance, surface::SurfaceInstance};

pub struct SurfaceManager {
    instance: SurfaceInstance,
    surface: SurfaceKHR,
}

impl SurfaceManager {
    pub fn init(instance: Arc<Instance>, window: &WindowManager) -> Result<Self, sdl3::Error> {
        let surface = window.create_surface(&instance)?;
        let surface_instance = SurfaceInstance::new(instance);
        Ok(Self {
            instance: surface_instance,
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
            self.instance.destroy_surface(self.surface);
        }
    }
}
