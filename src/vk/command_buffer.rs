use std::sync::{Arc, Weak};

use ash::vk;

use super::{command_pool::CommandPool, device::Device, error::fatal_vk_error};

#[derive(Debug, strum::Display, Clone, Copy)]
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
    command_pool: Arc<CommandPool>,
    device: Arc<Device>,
    command_buffer: vk::CommandBuffer,
    state: CommandBufferState,
}

impl CommandBuffer {
    pub fn new(
        command_pool: Arc<CommandPool>,
        device: Arc<Device>,
        command_buffer: vk::CommandBuffer,
    ) -> Self {
        CommandBuffer {
            command_pool,
            device,
            command_buffer,
            state: CommandBufferState::Initial,
        }
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
}
