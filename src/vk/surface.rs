use std::{fmt, sync::Arc};

use ash::vk::{self, PhysicalDevice, SurfaceKHR};

use crate::window::WindowManager;

use super::instance::{Instance, surface::SurfaceInstance};

pub struct PhysicalDeviceSurfaceInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
pub struct Surface {
    instance: SurfaceInstance,
    surface: SurfaceKHR,
}

impl Surface {
    pub fn init(instance: Arc<Instance>, window: &WindowManager) -> Result<Self, sdl3::Error> {
        let surface = window.create_surface(&instance)?;
        let surface_instance = SurfaceInstance::new(instance);
        let surface = Self {
            instance: surface_instance,
            surface,
        };

        log::info!("Created {:?}", surface);
        log::debug!(
            surface:?;
            "
{:?} Info:
instance: {:?}
",
            surface,
            surface.instance,
        );

        Ok(surface)
    }

    pub fn get_physical_device_surface_support(
        &self,
        device: PhysicalDevice,
        id: u32,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.instance
                .get_physical_device_surface_support(device, id, self.surface)
        }
    }
    pub fn get_physical_device_surface_info(
        &self,
        device: PhysicalDevice,
    ) -> Result<PhysicalDeviceSurfaceInfo, vk::Result> {
        unsafe {
            self.instance
                .get_physical_device_surface_info(device, self.surface)
        }
    }

    ///
    /// # Safety
    /// SurfaceKHR should not be destroyed via raw handle
    ///
    pub(in crate::vk) unsafe fn raw_handle(&self) -> SurfaceKHR {
        self.surface
    }
}

impl fmt::Debug for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Surface {:?}", self.surface)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_surface(self.surface);
        }
    }
}
