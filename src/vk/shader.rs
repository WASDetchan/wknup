use std::{ffi::CString, sync::Arc};

use ash::vk;

use super::device::Device;

#[derive(Clone)]
pub struct ShaderModule {
    device: Arc<Device>,
    pub shader: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(device: Arc<Device>, shader_raw: &[u32]) -> Self {
        Self {
            shader: unsafe { device.create_shader_module(shader_raw) },
            device,
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.shader);
        }
    }
}

#[derive(Clone)]
pub enum ShaderStage {
    Vertex,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
    Fragment,
    Compute,
    AllGraphics,
    All,
}

impl From<ShaderStage> for vk::ShaderStageFlags {
    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::TessellationControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
            ShaderStage::TessellationEvaluation => vk::ShaderStageFlags::TESSELLATION_EVALUATION,
            ShaderStage::Geometry => vk::ShaderStageFlags::GEOMETRY,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
            ShaderStage::AllGraphics => vk::ShaderStageFlags::ALL_GRAPHICS,
            ShaderStage::All => vk::ShaderStageFlags::ALL,
        }
    }
}

#[derive(Clone)]
pub struct ShaderStageInfo {
    pub shader: ShaderModule,
    pub stage: ShaderStage,
    pub entry_point: CString,
}

impl ShaderStageInfo {
    pub fn new(shader: ShaderModule, stage: ShaderStage, entry_point: String) -> Self {
        Self {
            stage,
            entry_point: CString::new(entry_point).expect("invalid entry_point"),
            shader,
        }
    }
    pub fn info(&self) -> vk::PipelineShaderStageCreateInfo<'_> {
        vk::PipelineShaderStageCreateInfo::default()
            .module(self.shader.shader)
            .name(self.entry_point.as_c_str())
            .stage(self.stage.clone().into())
    }
}
