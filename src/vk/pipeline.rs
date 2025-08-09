use std::collections::HashMap;

use crate::vk::shader::ShaderModule;

use super::{VulkanManager, shader::ShaderStageInfo};

pub struct PipelineManager {
    shader_stages: HashMap<String, ShaderStageInfo>,
}

impl PipelineManager {
    pub fn init(vulkan: &VulkanManager) -> Self {
        Self {
            shader_stages: HashMap::new(),
        }
    }

    pub fn add_stage(&mut self, name: String, stage: ShaderStageInfo) {
        self.shader_stages.insert(name, stage);
    }
}
