use ash::vk::{ColorSpaceKHR, Format, PresentModeKHR, SurfaceFormatKHR};

use crate::vk::physical_device::PhysicalDeviceSurfaceInfo;

pub fn check_surface_info(surface_info: PhysicalDeviceSurfaceInfo) -> bool {
    if choose_format(surface_info.formats).is_none()
        || choose_present_mode(surface_info.present_modes).is_none()
    {
        return false;
    }
}

fn choose_format(formats: Vec<SurfaceFormatKHR>) -> Option<SurfaceFormatKHR> {
    for format in formats {
        if format.format == Format::B8G8R8A8_SRGB
            && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
        {
            return format;
        }
    }
    None
}

fn choose_present_mode(modes: Vec<PresentModeKHR>) -> Option<PresentModeKHR> {
    for mode in modes {
        if mode == PresentModeKHR::FIFO {
            return mode;
        }
    }
    None
}
