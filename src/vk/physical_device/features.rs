use ash::{
    Instance,
    vk::{self, ExtendsDeviceCreateInfo},
};

#[derive(Default, Debug)]
pub struct FeaturesInfo {
    pub features: vk::PhysicalDeviceFeatures,
    pub vulkan_memory_model: bool,
    pub vulkan_memory_model_device_scope: bool,
    pub vulkan_memory_model_availability_visibility_chains: bool,
}
#[derive(Debug, thiserror::Error)]
#[error("not all required device features are available")]
pub struct MissingDeviceFeature;

impl FeaturesInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_features2(features2: PhysicalDeviceFeatures2) -> Self {
        let mut s = Self::default();
        let vulkan_memory_model_features = features2.vulkan_memory_model_features;
        let features2 = features2.features2;
        s.features = features2.features;
        s.vulkan_memory_model = vulkan_memory_model_features.vulkan_memory_model > 0;
        s.vulkan_memory_model_device_scope =
            vulkan_memory_model_features.vulkan_memory_model_device_scope > 0;
        s.vulkan_memory_model_availability_visibility_chains =
            vulkan_memory_model_features.vulkan_memory_model_availability_visibility_chains > 0;
        s
    }

    pub fn check_required(&self) -> Result<(), MissingDeviceFeature> {
        let required = Self::from_features2(PhysicalDeviceFeatures2::new_required());
        if (required.vulkan_memory_model && !self.vulkan_memory_model)
            || (required.vulkan_memory_model_device_scope && !self.vulkan_memory_model_device_scope)
            || (required.vulkan_memory_model_availability_visibility_chains
                && !self.vulkan_memory_model_availability_visibility_chains)
            || (required.features.robust_buffer_access > self.features.robust_buffer_access)
            || (required.features.full_draw_index_uint32 > self.features.full_draw_index_uint32)
            || (required.features.image_cube_array > self.features.image_cube_array)
            || (required.features.independent_blend > self.features.independent_blend)
            || (required.features.geometry_shader > self.features.geometry_shader)
            || (required.features.tessellation_shader > self.features.tessellation_shader)
            || (required.features.sample_rate_shading > self.features.sample_rate_shading)
            || (required.features.dual_src_blend > self.features.dual_src_blend)
            || (required.features.logic_op > self.features.logic_op)
            || (required.features.multi_draw_indirect > self.features.multi_draw_indirect)
            || (required.features.draw_indirect_first_instance
                > self.features.draw_indirect_first_instance)
            || (required.features.depth_clamp > self.features.depth_clamp)
            || (required.features.depth_bias_clamp > self.features.depth_bias_clamp)
            || (required.features.fill_mode_non_solid > self.features.fill_mode_non_solid)
            || (required.features.depth_bounds > self.features.depth_bounds)
            || (required.features.wide_lines > self.features.wide_lines)
            || (required.features.large_points > self.features.large_points)
            || (required.features.alpha_to_one > self.features.alpha_to_one)
            || (required.features.multi_viewport > self.features.multi_viewport)
            || (required.features.sampler_anisotropy > self.features.sampler_anisotropy)
            || (required.features.texture_compression_etc2 > self.features.texture_compression_etc2)
            || (required.features.texture_compression_astc_ldr
                > self.features.texture_compression_astc_ldr)
            || (required.features.texture_compression_bc > self.features.texture_compression_bc)
            || (required.features.occlusion_query_precise > self.features.occlusion_query_precise)
            || (required.features.pipeline_statistics_query
                > self.features.pipeline_statistics_query)
            || (required.features.vertex_pipeline_stores_and_atomics
                > self.features.vertex_pipeline_stores_and_atomics)
            || (required.features.fragment_stores_and_atomics
                > self.features.fragment_stores_and_atomics)
            || (required
                .features
                .shader_tessellation_and_geometry_point_size
                > self.features.shader_tessellation_and_geometry_point_size)
            || (required.features.shader_image_gather_extended
                > self.features.shader_image_gather_extended)
            || (required.features.shader_storage_image_extended_formats
                > self.features.shader_storage_image_extended_formats)
            || (required.features.shader_storage_image_multisample
                > self.features.shader_storage_image_multisample)
            || (required.features.shader_storage_image_read_without_format
                > self.features.shader_storage_image_read_without_format)
            || (required.features.shader_storage_image_write_without_format
                > self.features.shader_storage_image_write_without_format)
            || (required
                .features
                .shader_uniform_buffer_array_dynamic_indexing
                > self.features.shader_uniform_buffer_array_dynamic_indexing)
            || (required
                .features
                .shader_sampled_image_array_dynamic_indexing
                > self.features.shader_sampled_image_array_dynamic_indexing)
            || (required
                .features
                .shader_storage_buffer_array_dynamic_indexing
                > self.features.shader_storage_buffer_array_dynamic_indexing)
            || (required
                .features
                .shader_storage_image_array_dynamic_indexing
                > self.features.shader_storage_image_array_dynamic_indexing)
            || (required.features.shader_clip_distance > self.features.shader_clip_distance)
            || (required.features.shader_cull_distance > self.features.shader_cull_distance)
            || (required.features.shader_float64 > self.features.shader_float64)
            || (required.features.shader_int64 > self.features.shader_int64)
            || (required.features.shader_int16 > self.features.shader_int16)
            || (required.features.shader_resource_residency
                > self.features.shader_resource_residency)
            || (required.features.shader_resource_min_lod > self.features.shader_resource_min_lod)
            || (required.features.sparse_binding > self.features.sparse_binding)
            || (required.features.sparse_residency_buffer > self.features.sparse_residency_buffer)
            || (required.features.sparse_residency_image2_d
                > self.features.sparse_residency_image2_d)
            || (required.features.sparse_residency_image3_d
                > self.features.sparse_residency_image3_d)
            || (required.features.sparse_residency2_samples
                > self.features.sparse_residency2_samples)
            || (required.features.sparse_residency4_samples
                > self.features.sparse_residency4_samples)
            || (required.features.sparse_residency8_samples
                > self.features.sparse_residency8_samples)
            || (required.features.sparse_residency16_samples
                > self.features.sparse_residency16_samples)
            || (required.features.sparse_residency_aliased > self.features.sparse_residency_aliased)
            || (required.features.variable_multisample_rate
                > self.features.variable_multisample_rate)
            || (required.features.inherited_queries > self.features.inherited_queries)
        {
            Err(MissingDeviceFeature)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct PhysicalDeviceFeatures2<'a> {
    features2: Box<vk::PhysicalDeviceFeatures2<'a>>,
    vulkan_memory_model_features: Box<vk::PhysicalDeviceVulkanMemoryModelFeatures<'a>>,
}

impl<'a> Default for PhysicalDeviceFeatures2<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PhysicalDeviceFeatures2<'a> {
    pub fn new() -> Self {
        let mut vulkan_memory_model_features =
            Box::new(vk::PhysicalDeviceVulkanMemoryModelFeatures::default());

        let next_ptr = vulkan_memory_model_features.as_mut()
            as *mut vk::PhysicalDeviceVulkanMemoryModelFeatures;
        let features2 =
            Box::new(vk::PhysicalDeviceFeatures2::default().push_next(unsafe { &mut *next_ptr }));

        Self {
            vulkan_memory_model_features,
            features2,
        }
    }

    pub unsafe fn fill(&mut self, device: vk::PhysicalDevice, instance: &Instance) {
        unsafe {
            instance.get_physical_device_features2(device, self.features2.as_mut());
        }
    }

    pub fn dbg_ptr(&self) {
        dbg!(self.vulkan_memory_model_features.as_ref()
            as *const vk::PhysicalDeviceVulkanMemoryModelFeatures);
    }

    pub fn new_required() -> Self {
        let vulkan_memory_model_features =
            vk::PhysicalDeviceVulkanMemoryModelFeatures::default().vulkan_memory_model(true);
        let mut vulkan_memory_model_features = Box::new(vulkan_memory_model_features);

        let next_ptr = vulkan_memory_model_features.as_mut()
            as *mut vk::PhysicalDeviceVulkanMemoryModelFeatures;

        let features = vk::PhysicalDeviceFeatures::default().geometry_shader(true);
        let features2 = vk::PhysicalDeviceFeatures2::default()
            .features(features)
            .push_next(unsafe { &mut *next_ptr });
        let features2 = Box::new(features2);

        Self {
            vulkan_memory_model_features,
            features2,
        }
    }

    pub fn features(&self) -> vk::PhysicalDeviceFeatures {
        self.features2.as_ref().features
    }

    pub fn next(&self) -> impl ExtendsDeviceCreateInfo {
        *self.vulkan_memory_model_features.as_ref()
    }
}
