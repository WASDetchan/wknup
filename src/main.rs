use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut _vk_manager = wknup::vk::VulkanManager::init()?;
    // vk_manager.init_surface()?;
    Ok(())
}
