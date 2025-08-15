use std::{any::Any, sync::Arc};

use ash::vk;

use super::{
    command_pool::CommandPool,
    device::Device,
    error::fatal_vk_error,
    framebuffer::Framebuffer,
    pipeline::{GraphicsPipeline, render_pass::RenderPass},
};

#[derive(Default)]
pub struct DrawInfo {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

#[derive(Debug, strum::Display, Clone, Copy, PartialEq, Eq)]
pub enum CommandBufferState {
    Initial,
    Recording,
    Executable,
    Pending,
    Invalid,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid command buffer state: {0}")]
pub struct CommandBufferStateError(pub CommandBufferState);

pub struct CommandBuffer {
    _command_pool: Arc<CommandPool>,
    device: Arc<Device>,
    command_buffer: vk::CommandBuffer,
    state: CommandBufferState,
    markers: Vec<Arc<dyn Any>>,
}

impl CommandBuffer {
    pub fn new(
        command_pool: Arc<CommandPool>,
        device: Arc<Device>,
        command_buffer: vk::CommandBuffer,
    ) -> Self {
        CommandBuffer {
            _command_pool: command_pool,
            device,
            command_buffer,
            state: CommandBufferState::Initial,
            markers: Vec::new(),
        }
    }

    pub(in crate::vk) unsafe fn raw_handle(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub fn begin(&mut self) -> Result<(), CommandBufferStateError> {
        match self.state {
            CommandBufferState::Initial => (),
            CommandBufferState::Executable => (),
            state => return Err(CommandBufferStateError(state)),
        };
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device
                .raw_handle()
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap_or_else(|error| fatal_vk_error("failed to begin_command_buffer", error))
        }
        self.state = CommandBufferState::Recording;
        Ok(())
    }

    pub fn cmd_begin_render_pass(
        &mut self,
        render_pass: Arc<RenderPass>,
        framebuffer: Arc<Framebuffer>,
    ) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }

        let render_pass_begin = vk::RenderPassBeginInfo::default()
            .render_pass(unsafe { render_pass.raw_handle() })
            .framebuffer(unsafe { framebuffer.raw_handle() })
            .render_area(vk::Rect2D::default().extent(framebuffer.get_extent()))
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0f32, 0.0f32, 0.0f32, 1.0f32],
                },
            }]);

        unsafe {
            self.device.raw_handle().cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            );
        }

        self.markers.push(render_pass);
        self.markers.push(framebuffer);

        Ok(())
    }

    pub fn cmd_bind_graphics_pipeline(
        &mut self,
        pipeline: &GraphicsPipeline,
    ) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }

        unsafe {
            self.device.raw_handle().cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.raw_handle(),
            );
        }

        Ok(())
    }

    pub fn cmd_set_viewport(
        &mut self,
        viewport: vk::Viewport,
    ) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }
        unsafe {
            self.device
                .raw_handle()
                .cmd_set_viewport(self.command_buffer, 0, &[viewport]);
        }
        Ok(())
    }
    pub fn cmd_set_scissor(&mut self, scissor: vk::Rect2D) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }
        unsafe {
            self.device
                .raw_handle()
                .cmd_set_scissor(self.command_buffer, 0, &[scissor]);
        }
        Ok(())
    }

    pub fn cmd_draw(&mut self, draw_info: DrawInfo) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }
        let DrawInfo {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        } = draw_info;

        unsafe {
            self.device.raw_handle().cmd_draw(
                self.command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
        Ok(())
    }

    pub fn cmd_end_render_pass(&self) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }

        unsafe {
            self.device
                .raw_handle()
                .cmd_end_render_pass(self.command_buffer);
        }

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), CommandBufferStateError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferStateError(self.state));
        }

        unsafe {
            self.device
                .raw_handle()
                .end_command_buffer(self.command_buffer)
                .unwrap_or_else(|error| fatal_vk_error("failed to record command buffer", error));
        }
        self.state = CommandBufferState::Executable;

        Ok(())
    }
}
