use std::error::Error;

use sdl3::{
    self, Sdl, VideoSubsystem,
    video::{VkInstance, VkSurfaceKHR, Window},
};

pub struct WindowManager {
    _sdl_context: Sdl,
    _video_subsystem: VideoSubsystem,
    window: Window,
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
        }
    }
    pub fn create_vk_surface(&self, instance: VkInstance) -> Result<VkSurfaceKHR, Box<dyn Error>> {
        Ok(self.window.vulkan_create_surface(instance)?)
    }
    pub fn get_vk_extensions(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.window.vulkan_instance_extensions()?)
    }
}
