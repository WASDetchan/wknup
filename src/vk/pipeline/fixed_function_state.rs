use ash::vk;

pub struct FixedFuctionState {
    dynamic_states: Vec<vk::DynamicState>,
    color_blend_attachment_states: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl Default for FixedFuctionState {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedFuctionState {
    pub fn new() -> Self {
        Self {
            dynamic_states: vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR],
            color_blend_attachment_states: vec![
                vk::PipelineColorBlendAttachmentState::default().color_write_mask(
                    vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A,
                ),
            ],
        }
    }

    pub fn get_dynamic_state(&self) -> vk::PipelineDynamicStateCreateInfo<'_> {
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&self.dynamic_states)
    }
    pub fn get_vertex_input_state(&self) -> vk::PipelineVertexInputStateCreateInfo<'_> {
        vk::PipelineVertexInputStateCreateInfo::default()
    }
    pub fn get_input_assembly_state(&self) -> vk::PipelineInputAssemblyStateCreateInfo<'_> {
        vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    }
    pub fn get_viewport_state(&self) -> vk::PipelineViewportStateCreateInfo<'_> {
        vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1)
    }
    pub fn get_rasterization_state(&self) -> vk::PipelineRasterizationStateCreateInfo<'_> {
        vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0f32)
    }
    pub fn get_multisample_state(&self) -> vk::PipelineMultisampleStateCreateInfo<'_> {
        vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    }

    pub fn get_color_blend_state(&self) -> vk::PipelineColorBlendStateCreateInfo<'_> {
        vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&self.color_blend_attachment_states)
    }
}
