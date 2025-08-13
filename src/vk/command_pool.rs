use std::sync::Arc;

use ash::vk;

use super::{device::Device, error::fatal_vk_error};

#[derive(Debug, thiserror::Error)]
pub enum CommandPoolCreationError {
    #[error("queue faily index {0} is out of range 0..{1}")]
    InvalidQueueFamily(usize, usize),
}

pub struct CommandPool {
    device: Arc<Device>,
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(
        device: Arc<Device>,
        queue_family_index: u32,
    ) -> Result<Self, CommandPoolCreationError> {
        if queue_family_index as usize >= device.get_physical_device_choice().queue_counts.len() {
            return Err(CommandPoolCreationError::InvalidQueueFamily(
                queue_family_index as usize,
                device.get_physical_device_choice().queue_counts.len(),
            ));
        }
        let create_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(queue_family_index);
        let command_pool = unsafe { device.raw_handle().create_command_pool(&create_info, None) }
            .unwrap_or_else(|error| fatal_vk_error("failed to create_command_pool", error));

        Ok(Self {
            device,
            command_pool,
        })
    }
}
