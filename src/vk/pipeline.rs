mod fixed_function_state;
pub mod layout;
pub mod render_pass;
use ash::vk;
use layout::PipelineLayout;
use render_pass::RenderPass;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Weak},
};

use fixed_function_state::FixedFuctionState;

use crate::vk::{
    command_buffer::CommandBuffer,
    command_pool::CommandPool,
    device::Device,
    framebuffer::Framebuffer,
    shader::{MissingShaderStageError, ShaderStage, ShaderStageInfo},
    swapchain::Swapchain,
};

use super::command_buffer::DrawInfo;

pub struct GraphicsPipelineBuilder {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    shader_stages: HashMap<String, ShaderStageInfo>,
}

impl GraphicsPipelineBuilder {
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Self {
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
    fn require_stage(&self, stage: ShaderStage) -> Result<(), MissingShaderStageError> {
        if !self
            .shader_stages
            .iter()
            .any(|(_, info)| info.stage() == stage)
        {
            Err(MissingShaderStageError::new(stage))
        } else {
            Ok(())
        }
    }
    pub fn build(self) -> Result<Arc<GraphicsPipeline>, Box<dyn Error>> {
        self.require_stage(ShaderStage::Vertex)?;
        self.require_stage(ShaderStage::Fragment)?;
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

        let render_pass = Arc::new(RenderPass::new(
            Arc::clone(&self.device),
            Arc::clone(&self.swapchain),
        )?);

        let layout = PipelineLayout::new(Arc::clone(&self.device));

        let stages: Vec<_> = self.shader_stages.values().map(|val| val.info()).collect();

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

        let framebuffers = self.swapchain.create_framebuffers(render_pass.clone());

        let pipeline = unsafe { self.device.create_graphics_pipeline(pipeline_create_info)? };

        Ok(Arc::new_cyclic(|weak_self| GraphicsPipeline {
            weak_self: Weak::clone(weak_self),
            device: self.device,
            swapchain: self.swapchain,
            shader_stages: self.shader_stages,
            layout,
            render_pass,
            pipeline,
            framebuffers,
        }))
    }
}

#[allow(dead_code)]
pub struct GraphicsPipeline {
    weak_self: Weak<Self>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    shader_stages: HashMap<String, ShaderStageInfo>,
    layout: PipelineLayout,
    render_pass: Arc<RenderPass>,
    pipeline: vk::Pipeline,
    framebuffers: Vec<Arc<Framebuffer>>,
}

impl GraphicsPipeline {
    pub fn create_framebuffers(&mut self) {
        self.framebuffers = self
            .swapchain
            .create_framebuffers(Arc::clone(&self.render_pass));
    }

    pub fn create_command_buffers(&self, command_pool: CommandPool) -> Vec<CommandBuffer> {
        let mut command_buffer = command_pool.allocate_command_buffer();
        command_buffer.begin().unwrap();
        command_buffer
            .cmd_begin_render_pass(Arc::clone(&self.render_pass), todo!())
            .unwrap();
        command_buffer
            .cmd_bind_graphics_pipeline(self.weak_self.upgrade().unwrap())
            .unwrap();
        let (viewport, scissor) = self.swapchain.make_viewport().unwrap();
        command_buffer.cmd_set_viewport(viewport).unwrap();
        command_buffer.cmd_set_scissor(scissor).unwrap();
        command_buffer
            .cmd_draw(DrawInfo {
                vertex_count: 3,
                instance_count: 1,
                ..Default::default()
            })
            .unwrap();
        command_buffer.end().unwrap();
        Vec::new()
    }

    pub(in crate::vk) unsafe fn raw_handle(&self) -> vk::Pipeline {
        self.pipeline
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline);
        }
    }
}
