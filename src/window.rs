use crate::vk::instance::InstanceManager;
use std::{error::Error, sync::Arc};

use ash::vk::SurfaceKHR;
use sdl3::{self, Sdl, VideoSubsystem, video::Window};

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

    pub fn create_surface(
        &self,
        instance: &Arc<InstanceManager>,
    ) -> Result<SurfaceKHR, Box<dyn Error>> {
        let surface = instance.create_surface(&self.window)?;
        Ok(surface)
    }

    pub fn get_vk_extensions(&self) -> Result<Vec<String>, sdl3::Error> {
        self.window.vulkan_instance_extensions()
    }
}
