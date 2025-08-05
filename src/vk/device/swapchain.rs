use std::{error::Error, sync::Arc};

use ash::vk::{
    ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, ImageUsageFlags, PresentModeKHR,
    SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR,
    SwapchainCreateInfoKHR, SwapchainKHR,
};

use crate::vk::physical_device::{PhysicalDeviceSurfaceInfo, QueueFamilyIndices};

use super::DeviceManager;

pub fn check_surface_info(surface_info: PhysicalDeviceSurfaceInfo) -> bool {
    if choose_format(surface_info.formats).is_none()
        || choose_present_mode(surface_info.present_modes).is_none()
    {
        return false;
    }
    true
}

fn choose_format(formats: Vec<SurfaceFormatKHR>) -> Option<SurfaceFormatKHR> {
    for format in formats {
        if format.format == Format::B8G8R8A8_SRGB
            && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
        {
            return Some(format);
        }
    }
    None
}

fn choose_present_mode(modes: Vec<PresentModeKHR>) -> Option<PresentModeKHR> {
    for mode in modes {
        if mode == PresentModeKHR::FIFO {
            return Some(mode);
        }
    }
    None
}

fn choose_swap_extent(capabilities: SurfaceCapabilitiesKHR) -> Extent2D {
    if capabilities.current_extent.height != u32::MAX {
        return capabilities.current_extent;
    }
    todo!("swap extent was not set");
}

fn choose_image_count(capabilities: SurfaceCapabilitiesKHR) -> u32 {
    let image_count = capabilities.min_image_count + 1;
    if capabilities.max_image_count != 0 && capabilities.max_image_count < image_count {
        capabilities.max_image_count
    } else {
        image_count
    }
}

fn choose_transform(capabilities: SurfaceCapabilitiesKHR) -> SurfaceTransformFlagsKHR {
    capabilities.current_transform
}

pub struct SwapchainManager {
    swapchain: Option<SwapchainKHR>,
    device: Arc<DeviceManager>,
}

impl SwapchainManager {
    pub fn new(device: Arc<DeviceManager>) -> Self {
        Self {
            swapchain: None,
            device,
        }
    }
    pub fn create_swapchain(
        &mut self,
        surface: SurfaceKHR,
        surface_info: PhysicalDeviceSurfaceInfo,
        queue_family_indices: QueueFamilyIndices,
    ) -> Result<(), Box<dyn Error>> {
        let graphic = queue_family_indices.graphics.unwrap();
        let present = queue_family_indices.present.unwrap();
        let indices = [graphic as u32, present as u32];

        let capabilities = surface_info.capabilities;
        let format = choose_format(surface_info.formats).unwrap();
        let mut swapchain_info = SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(choose_image_count(capabilities))
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(choose_swap_extent(capabilities))
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(choose_transform(capabilities))
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(choose_present_mode(surface_info.present_modes).unwrap())
            .clipped(true);

        if let Some(swapchain) = self.swapchain.take() {
            swapchain_info = swapchain_info.old_swapchain(swapchain);
        }

        if graphic == present {
            swapchain_info = swapchain_info.image_sharing_mode(SharingMode::EXCLUSIVE)
        } else {
            swapchain_info = swapchain_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&indices);
        }
        self.swapchain = Some(self.device.create_swapchain(&swapchain_info)?);
        Ok(())
    }
}

impl Drop for SwapchainManager {
    fn drop(&mut self) {
        if self.swapchain.is_some() {
            unsafe {
                self.device
                    .destroy_swapchain(self.swapchain.unwrap())
                    .unwrap();
            }
        }
    }
}
