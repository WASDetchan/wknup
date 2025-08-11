use ash::vk;
use std::{collections::HashMap, sync::Arc};

use crate::vk::device::swapchain::Swapchain;

use super::{Vulkan, device::Device, shader::ShaderStageInfo};

pub struct GraphicsPipelineBuilder {
    device: Arc<Device>,
    swapchain: Swapchain,
    shader_stages: HashMap<String, ShaderStageInfo>,
    vertex_stage: String,
    fragment_stage: String,
}

impl GraphicsPipelineBuilder {
    pub fn new(vulkan: Arc<Vulkan>, swapchain: Swapchain) -> Self {
        let device = vulkan.get_device();
        Self {
            device,
            swapchain,
            shader_stages: HashMap::new(),
            vertex_stage: String::new(),
            fragment_stage: String::new(),
        }
    }
    const DYNAMIC_STATES: [vk::DynamicState; 2] =
        [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    pub fn add_stage(mut self, name: String, stage: ShaderStageInfo) -> Self {
        self.shader_stages.insert(name, stage);
        self
    }
    pub fn vertex_stage(mut self, stage: String) -> Self {
        self.vertex_stage = stage;
        self
    }
    pub fn fragment_stage(mut self, stage: String) -> Self {
        self.fragment_stage = stage;
        self
    }
    pub fn build(self) -> Result<GraphicsPipeline, Box<dyn std::error::Error>> {
        let dynamic_states = Self::DYNAMIC_STATES;
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0f32);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment_state = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&color_blend_attachment_state);

        let layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe { self.device.create_pipeline_layout(layout_info) };

        let attachment_description = [vk::AttachmentDescription::default()
            .samples(vk::SampleCountFlags::TYPE_1)
            .format(self.swapchain.get_format().format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let attachment_reference = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let subpass_description = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&attachment_reference)];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment_description)
            .subpasses(&subpass_description);

        let render_pass = unsafe { self.device.create_render_pass(&render_pass_info)? };
        let vert = self.shader_stages.get(&self.vertex_stage).unwrap().clone();
        let frag = self
            .shader_stages
            .get(&self.fragment_stage)
            .unwrap()
            .clone();
        let stages = vec![
            vk::PipelineShaderStageCreateInfo::default()
                .stage(frag.stage.into())
                .module(frag.shader.shader)
                .name(frag.entry_point.as_c_str()),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vert.stage.into())
                .module(vert.shader.shader)
                .name(vert.entry_point.as_c_str()),
        ];
        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe { self.device.create_graphics_pipeline(pipeline_create_info)? };

        Ok(GraphicsPipeline {
            device: self.device,
            shader_stages: self.shader_stages,
            layout: pipeline_layout,
            render_pass,
            pipeline,
        })
    }
}

#[allow(dead_code)]
pub struct GraphicsPipeline {
    device: Arc<Device>,
    shader_stages: HashMap<String, ShaderStageInfo>,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
}

impl GraphicsPipeline {}
