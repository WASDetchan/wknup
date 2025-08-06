use std::error::Error;

use wknup::window::WindowManager;

fn main() -> Result<(), Box<dyn Error>> {
    let window = WindowManager::init();
    start(&window)?;
    Ok(())
}

#[tokio::main]
async fn start(window: &WindowManager) -> Result<(), Box<dyn Error>> {
    let mut _vk_manager = wknup::vk::VulkanManager::init(window)?;
    Ok(())
}
