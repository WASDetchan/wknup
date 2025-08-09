

use crate::vk::shader::ShaderModule;

use super::VulkanManager;

pub struct PipelineManager {
    shader: ShaderModule,
}

impl PipelineManager {
    pub fn init(vulkan: &VulkanManager, shader: ShaderModule) -> Self {
        Self { shader }
    }
}
