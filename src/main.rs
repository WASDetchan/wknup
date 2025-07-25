use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut vk_manager = wknup::vk::VulkanManager::init()?;
    vk_manager.check_extensions(
        vec!["VK_KHR_device_group_creation"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect(),
    )?;
    vk_manager.init_surface()?;
    Ok(())
}
