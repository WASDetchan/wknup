use std::sync::{Arc, Weak};

use ash::vk;

use super::{command_buffer::CommandBuffer, device::Device, error::fatal_vk_error};

#[derive(Debug, thiserror::Error)]
pub enum CommandPoolCreationError {
    #[error("queue faily index {0} is out of range 0..{1}")]
    InvalidQueueFamily(usize, usize),
}

pub struct CommandPool {
    weak_self: Weak<Self>,
    device: Arc<Device>,
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(
        device: Arc<Device>,
        queue_family_index: u32,
    ) -> Result<Arc<Self>, CommandPoolCreationError> {
        if queue_family_index as usize >= device.get_queue_family_count() {
            return Err(CommandPoolCreationError::InvalidQueueFamily(
                queue_family_index as usize,
                device.get_queue_family_count(),
            ));
        }
        let create_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(queue_family_index);
        let command_pool = unsafe { device.raw_handle().create_command_pool(&create_info, None) }
            .unwrap_or_else(|error| fatal_vk_error("failed to create_command_pool", error));

        Ok(Arc::new_cyclic(|weak_self| Self {
            weak_self: Weak::clone(weak_self),
            device,
            command_pool,
        }))
    }

    pub fn allocate_command_buffer(&self) -> CommandBuffer {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffer = unsafe {
            self.device
                .raw_handle()
                .allocate_command_buffers(&allocate_info)
                .unwrap_or_else(|error| fatal_vk_error("failed to allocate_command_buffer", error))
                [0]
        };
        CommandBuffer::new(
            self.weak_self.upgrade().unwrap(),
            self.device.clone(),
            command_buffer,
        )
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw_handle()
                .destroy_command_pool(self.command_pool, None);
        }
    }
}
