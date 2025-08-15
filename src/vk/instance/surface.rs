use std::{fmt, sync::Arc};

use ash::{
    khr,
    vk::{self, SurfaceKHR},
};

use crate::vk::surface::PhysicalDeviceSurfaceInfo;

use super::Instance;

pub struct SurfaceInstance {
    instance: Arc<Instance>, // Has to have a reference to the original Instance for ensuring
    // correctness of lifetimes
    surface_khr_instance: khr::surface::Instance,
}

impl SurfaceInstance {
    pub fn new(instance: Arc<Instance>) -> Self {
        let surface_khr_instance = unsafe { instance.make_surface_instance() };
        let s = Self {
            instance,
            surface_khr_instance,
        };
        log::info!("Created {:?}", s);
        s
    }
    pub unsafe fn get_physical_device_surface_support(
        &self,
        device: vk::PhysicalDevice,
        id: u32,
        surface: SurfaceKHR,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.surface_khr_instance
                .get_physical_device_surface_support(device, id, surface)
        }
    }

    pub unsafe fn get_physical_device_surface_info(
        &self,
        device: vk::PhysicalDevice,
        surface: SurfaceKHR,
    ) -> Result<PhysicalDeviceSurfaceInfo, vk::Result> {
        unsafe {
            let capabilities = self
                .surface_khr_instance
                .get_physical_device_surface_capabilities(device, surface)?;
            let formats = self
                .surface_khr_instance
                .get_physical_device_surface_formats(device, surface)?;
            let present_modes = self
                .surface_khr_instance
                .get_physical_device_surface_present_modes(device, surface)?;
            Ok(PhysicalDeviceSurfaceInfo {
                capabilities,
                formats,
                present_modes,
            })
        }
    }

    pub unsafe fn destroy_surface(&self, surface: SurfaceKHR) {
        unsafe {
            self.surface_khr_instance.destroy_surface(surface, None);
        }
    }
}
impl fmt::Debug for SurfaceInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SurfaceInstance of {:?}", self.instance)
    }
}
