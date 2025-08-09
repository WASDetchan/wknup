use ash::vk;
use std::{collections::HashMap, sync::RwLock};

use crate::vk::shader::ShaderModule;

use super::{device::swapchain::SwapchainManager, shader::ShaderStageInfo, VulkanManager};

pub struct PipelineManager {
    vulkan: Arc<VulkanManager>,
    swapchain: RwLock<SwapchainManager>
    shader_stages: HashMap<String, ShaderStageInfo>,

}

impl PipelineManager {
    pub fn init(vulkan: Arc<VulkanManager>) -> Self {
        Self {
            shader_stages: HashMap::new(),
        }
    }

    pub fn add_stage(&mut self, name: String, stage: ShaderStageInfo) {
        self.shader_stages.insert(name, stage);
    }

    pub fn init_fixed_fuctions_info(&mut self) {
        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let (viewport, scissor) = self.swapchain.read().unwrap().make_viewport()?;
        let viewport_state = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).line_width(1.0f32);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::default().color_write_mask(vk::ColorComponentFlags::R || vk::ColorComponentFlags::G || vk::ColorComponentFlags::B || vk::ColorComponentFlags::A);
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default().attachments(&[color_blend_attachment_state]);

        let layout = vk::PipelineLayout::default();

    }
}
