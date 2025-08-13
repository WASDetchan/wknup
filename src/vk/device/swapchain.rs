use std::{error::Error, sync::Arc};

use ash::vk::{
    self, ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, ImageAspectFlags,
    ImageUsageFlags, ImageViewCreateInfo, PresentModeKHR, SharingMode, SurfaceCapabilitiesKHR,
    SurfaceFormatKHR, SurfaceTransformFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR,
};

use crate::vk::{
    framebuffer::Framebuffer, pipeline::render_pass::RenderPass,
    surface::PhysicalDeviceSurfaceInfo, surface::SurfaceManager,
};

use super::Device;
use thiserror;

#[derive(Debug, thiserror::Error)]
#[error("the swapchain SwapchainManager currently has is missing or invalid")]
pub struct InvalidSwapchainError;

pub fn check_surface_info(surface_info: PhysicalDeviceSurfaceInfo) -> bool {
    if choose_format(surface_info.formats).is_none()
        || choose_present_mode(surface_info.present_modes).is_none()
    {
        return false;
    }
    true
}

fn choose_format(formats: Vec<SurfaceFormatKHR>) -> Option<SurfaceFormatKHR> {
    formats.into_iter().find(|&format| {
        format.format == Format::B8G8R8A8_SRGB
            && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
    })
}

fn choose_present_mode(modes: Vec<PresentModeKHR>) -> Option<PresentModeKHR> {
    modes.into_iter().find(|&mode| mode == PresentModeKHR::FIFO)
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

pub struct Swapchain {
    device: Arc<Device>,
    _surface: Arc<SurfaceManager>,
    swapchain_khr: SwapchainKHR,
    extent: Extent2D,
    format: SurfaceFormatKHR,
    _present_mode: PresentModeKHR,
    _images: Vec<vk::Image>,
    views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn make_viewport(&self) -> Result<(vk::Viewport, vk::Rect2D), InvalidSwapchainError> {
        let Extent2D { width, height } = self.extent;
        let viewport = vk::Viewport::default()
            .width(width as f32)
            .height(height as f32)
            .max_depth(1.0f32);
        let scissor = vk::Rect2D::default().extent(self.extent);
        Ok((viewport, scissor))
    }
    pub fn get_format(&self) -> SurfaceFormatKHR {
        self.format
    }
    pub fn create_framebuffers(&self, render_pass: Arc<RenderPass>) -> Vec<Framebuffer> {
        self.views
            .iter()
            .map(|view| {
                let attachments = [view.clone()];
                let create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(unsafe { render_pass.raw_handle() })
                    .attachments(&attachments)
                    .height(self.extent.height)
                    .width(self.extent.width)
                    .layers(1);
                let framebuffer = unsafe { self.device.create_framebuffer(&create_info) };
                Framebuffer::new(
                    Arc::clone(&self.device),
                    Arc::clone(&render_pass),
                    framebuffer,
                )
            })
            .collect()
    }
}
impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_swapchain(self.swapchain_khr).unwrap();
        }
        for _ in 0..self.views.len() {
            unsafe {
                self.device.destroy_image_view(self.views.pop().unwrap());
            }
        }
    }
}
pub struct SwapchainManager {
    device: Arc<Device>,
    surface: Arc<SurfaceManager>,
}

impl SwapchainManager {
    pub fn new(device: Arc<Device>, surface: Arc<SurfaceManager>) -> Self {
        Self { device, surface }
    }
    pub fn create_swapchain(&self) -> Result<Swapchain, Box<dyn Error>> {
        let surface_info = self.device.get_surface_info()?;
        let queue_family_chooser = self
            .device
            .get_physical_device_choice()
            .queue_family_chooser;

        let graphic = queue_family_chooser.graphics.unwrap();
        let present = queue_family_chooser.present.unwrap();
        let indices = [graphic as u32, present as u32];

        let capabilities = surface_info.capabilities;

        let format = choose_format(surface_info.formats).unwrap();
        let extent = choose_swap_extent(capabilities);
        let present_mode = choose_present_mode(surface_info.present_modes).unwrap();

        let mut swapchain_info = SwapchainCreateInfoKHR::default()
            .surface(unsafe { self.surface.raw_handle() })
            .min_image_count(choose_image_count(capabilities))
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(choose_transform(capabilities))
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        if graphic == present {
            swapchain_info = swapchain_info.image_sharing_mode(SharingMode::EXCLUSIVE)
        } else {
            swapchain_info = swapchain_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&indices);
        }
        let swapchain_khr = self.device.create_swapchain(&swapchain_info)?;
        let images = unsafe { self.device.get_swapchain_images(swapchain_khr) }?;

        let view_info = ImageViewCreateInfo::default()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .level_count(1)
                    .layer_count(1)
                    .aspect_mask(ImageAspectFlags::COLOR),
            );

        let views = images
            .iter()
            .map(|image| {
                let info = view_info.image(*image);
                unsafe { self.device.create_image_view(&info) }
            })
            .collect();

        Ok(Swapchain {
            _surface: Arc::clone(&self.surface),
            device: Arc::clone(&self.device),
            swapchain_khr,
            _images: images,
            views,
            format,
            _present_mode: present_mode,
            extent,
        })
    }
}
