use std::sync::Arc;

use crate::vk::{device::Device, swapchain::Swapchain};
use ash::vk;

pub struct RenderPass {
    device: Arc<Device>,
    _swapchain: Arc<Swapchain>,
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Result<Self, vk::Result> {
        let attachment_description = [vk::AttachmentDescription::default()
            .samples(vk::SampleCountFlags::TYPE_1)
            .format(swapchain.get_format().format)
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

        let dependency = [vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment_description)
            .subpasses(&subpass_description)
            .dependencies(&dependency);

        let render_pass = unsafe { device.create_render_pass(&render_pass_info)? };

        Ok(Self {
            device,
            _swapchain: swapchain,
            render_pass,
        })
    }

    pub(in crate::vk) unsafe fn raw_handle(&self) -> vk::RenderPass {
        self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.render_pass);
        }
    }
}
