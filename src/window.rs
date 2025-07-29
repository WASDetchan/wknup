use crate::vk::instance::InstanceManager;
use std::{error::Error, sync::Arc};

use sdl3::{
    self, Sdl, VideoSubsystem,
    video::{VkSurfaceKHR, Window},
};

pub struct WindowManager {
    _sdl_context: Sdl,
    _video_subsystem: VideoSubsystem,
    window: Window,
    surface: Option<VkSurfaceKHR>,
    // surface_instance: Option<khr::surface::Instance>,
    instance: Option<Arc<InstanceManager>>,
}

impl WindowManager {
    pub fn init() -> Self {
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Test window", 800, 600)
            .position_centered()
            .vulkan()
            .build()
            .unwrap();

        Self {
            _sdl_context: sdl_context,
            _video_subsystem: video_subsystem,
            window,
            surface: None,
            instance: None,
        }
    }

    pub fn init_surface(&mut self, instance: Arc<InstanceManager>) -> Result<(), Box<dyn Error>> {
        let surface = Some(instance.create_surface(&self.window)?);
        self.surface = surface;
        self.instance = Some(instance);
        Ok(())
    }
    // pub fn create_vk_surface(&self, instance: VkInstance) -> Result<VkSurfaceKHR, Box<dyn Error>> {
    //     Ok(self.window.vulkan_create_surface(instance)?)
    // }
    pub fn get_vk_extensions(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.window.vulkan_instance_extensions()?)
    }
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        if let Some(surface) = self.surface {
            self.instance
                .as_ref()
                .expect("instance is initalized before surface")
                .destroy_surface(surface)
                .expect("instance is initalized before surface");
        }
    }
}
