mod layout;
mod render_pass;
use ash::vk::{self};
use layout::PipelineLayout;
use render_pass::RenderPass;
use std::{collections::HashMap, sync::Arc};

use crate::vk::device::swapchain::Swapchain;

use super::{Vulkan, device::Device, shader::ShaderStageInfo};

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

    pub fn get_dynamic_state(&self) -> vk::PipelineDynamicStateCreateInfo {
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&self.dynamic_states)
    }
    pub fn get_vertex_input_state(&self) -> vk::PipelineVertexInputStateCreateInfo {
        vk::PipelineVertexInputStateCreateInfo::default()
    }
    pub fn get_input_assembly_state(&self) -> vk::PipelineInputAssemblyStateCreateInfo {
        vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
    }
    pub fn get_viewport_state(&self) -> vk::PipelineViewportStateCreateInfo {
        vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1)
    }
    pub fn get_rasterization_state(&self) -> vk::PipelineRasterizationStateCreateInfo {
        vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0f32)
    }
    pub fn get_multisample_state(&self) -> vk::PipelineMultisampleStateCreateInfo {
        vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    }

    pub fn get_color_blend_state(&self) -> vk::PipelineColorBlendStateCreateInfo {
        vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(&self.color_blend_attachment_states)
    }
}

pub struct GraphicsPipelineBuilder {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    shader_stages: HashMap<String, ShaderStageInfo>,
}

impl GraphicsPipelineBuilder {
    pub fn new(vulkan: Arc<Vulkan>, swapchain: Arc<Swapchain>) -> Self {
        let device = vulkan.get_device();
        Self {
            device,
            swapchain,
            shader_stages: HashMap::new(),
        }
    }
    pub fn add_stage(mut self, name: String, stage: ShaderStageInfo) -> Self {
        self.shader_stages.insert(name, stage);
        self
    }
    pub fn build(self) -> Result<GraphicsPipeline, Box<dyn std::error::Error>> {
        let fixed_function_state = FixedFuctionState::new();
        let (
            vertex_input_state,
            input_assembly_state,
            viewport_state,
            rasterization_state,
            multisample_state,
            color_blend_state,
            dynamic_state,
        ) = (
            fixed_function_state.get_vertex_input_state(),
            fixed_function_state.get_input_assembly_state(),
            fixed_function_state.get_viewport_state(),
            fixed_function_state.get_rasterization_state(),
            fixed_function_state.get_multisample_state(),
            fixed_function_state.get_color_blend_state(),
            fixed_function_state.get_dynamic_state(),
        );

        let render_pass = RenderPass::new(Arc::clone(&self.device), Arc::clone(&self.swapchain))?;

        let layout = PipelineLayout::new(Arc::clone(&self.device));

        let stages: Vec<_> = self
            .shader_stages
            .iter()
            .map(|(key, val)| val.info())
            .collect();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(unsafe { layout.raw_handle() })
            .render_pass(unsafe { render_pass.raw_handle() })
            .subpass(0);

        let pipeline = unsafe { self.device.create_graphics_pipeline(pipeline_create_info)? };

        Ok(GraphicsPipeline {
            device: self.device,
            swapchain: self.swapchain,
            shader_stages: self.shader_stages,
            layout,
            render_pass,
            pipeline,
        })
    }
}

#[allow(dead_code)]
pub struct GraphicsPipeline {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    shader_stages: HashMap<String, ShaderStageInfo>,
    layout: PipelineLayout,
    render_pass: RenderPass,
    pipeline: vk::Pipeline,
}

impl GraphicsPipeline {}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline);
        }
    }
}
