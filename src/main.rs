use std::error::Error;

use wknup::window::WindowManager;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Non async");
    let window = WindowManager::init();
    println!("Created window");
    start(&window)?;
    println!("Back to sync");
    Ok(())
}

#[tokio::main]
async fn start(window: &WindowManager) -> Result<(), Box<dyn Error>> {
    println!("Async");
    let mut _vk_manager = wknup::vk::VulkanManager::init(window)?;
    Ok(())
}
