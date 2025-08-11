use ash::vk;
use std::{collections::HashMap, sync::Arc};

use crate::vk::device::swapchain::Swapchain;

use super::{Vulkan, shader::ShaderStageInfo};

#[allow(dead_code)]
pub struct PipelineManager {
    vulkan: Arc<Vulkan>,
    swapchain: Swapchain,
    shader_stages: HashMap<String, ShaderStageInfo>,
}

impl PipelineManager {
    pub fn init(vulkan: Arc<Vulkan>, swapchain: Swapchain) -> Self {
        Self {
            shader_stages: HashMap::new(),
            vulkan,
            swapchain,
        }
    }

    pub fn add_stage(&mut self, name: String, stage: ShaderStageInfo) {
        self.shader_stages.insert(name, stage);
    }
    pub fn init_fixed_fuctions_info(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let _dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let _vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default();

        let _input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let (_viewport, _scissor) = self.swapchain.make_viewport()?;
        let _viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let _rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0f32);

        let _multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            );
        let _color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&[color_blend_attachment_state]);

        let _layout = vk::PipelineLayout::default();

        Ok(())
    }
}
