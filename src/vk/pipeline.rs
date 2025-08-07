use std::sync::Arc;

use super::VulkanManager;

struct PipelineManager {
    vulkan: Arc<VulkanManager>,
}

impl PipelineManager {
    fn init(vulkan: Arc<VulkanManager>) -> Self {
        Self { vulkan }
    }
}
