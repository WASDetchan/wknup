use std::{error::Error, sync::Arc, vec::IntoIter};

use ash::{
    Device,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceProperties, PhysicalDeviceType, Queue, QueueFamilyProperties, QueueFlags,
        SurfaceKHR,
    },
};

use super::instance::InstanceManager;

type QFFilter = Arc<dyn Fn(PhysicalDevice, usize, &QueueFamilyProperties) -> bool>;
#[derive(Clone)]
struct QueueFamilyIndicies {
    graphics: Option<usize>,
    present: Option<usize>,
    graphics_filter: QFFilter,
    present_filter: QFFilter,
}

struct Queues {
    graphics: Queue,
    present: Queue,
}

impl QueueFamilyIndicies {
    fn new(graphics_filter: QFFilter, present_filter: QFFilter) -> Self {
        Self {
            graphics: None,
            present: None,
            graphics_filter,
            present_filter,
        }
    }
    fn try_queue(
        &mut self,
        physical_device: PhysicalDevice,
        id: usize,
        props: &QueueFamilyProperties,
    ) {
        if (self.graphics_filter)(physical_device, id, props) {
            self.graphics = Some(id);
        };
        if (self.present_filter)(physical_device, id, props) {
            self.present = Some(id);
        }
    }
    fn fill(&mut self, instance: &Arc<InstanceManager>, physical_device: PhysicalDevice) {
        Self::iterate_physical_device_queue_families(instance, physical_device)
            .enumerate()
            .for_each(|(id, prop)| self.try_queue(physical_device, id, &prop));
    }
    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
    fn iterate_physical_device_queue_families(
        instance: &Arc<InstanceManager>,
        physical_device: PhysicalDevice,
    ) -> IntoIter<QueueFamilyProperties> {
        instance
            .get_physical_device_queue_family_properties(physical_device)
            .unwrap()
            .into_iter()
    }
}

pub struct PhysicalDeviceInfo {
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
}

pub struct DeviceManager {
    _physical_device: PhysicalDevice,
    _queue_families: QueueFamilyIndicies,
    instance: Arc<InstanceManager>,
    device: Option<Device>,
    queue: Queue,
}
impl DeviceManager {
    fn iterate_physical_devices(
        instance: Arc<InstanceManager>,
    ) -> Result<IntoIter<PhysicalDevice>, Box<dyn Error>> {
        Ok(instance.enumerate_physical_devices()?.into_iter())
    }

    fn rate_physical_device(
        instance: &Arc<InstanceManager>,
        device: PhysicalDevice,
        mut qfi: QueueFamilyIndicies,
    ) -> i32 {
        let info = instance.get_physical_device_info(device).unwrap();
        let props = info.properties;
        let features = info.features;
        qfi.fill(instance, device);
        ((props.device_type == PhysicalDeviceType::DISCRETE_GPU
            || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
            && (features.geometry_shader == 1)
            && qfi.is_complete()) as i32
    }

    pub fn init(
        instance: Arc<InstanceManager>,
        surface_khr: SurfaceKHR,
    ) -> Result<Self, Box<dyn Error>> {
        let d_instance = instance.clone();
        let mut qfi = QueueFamilyIndicies::new(
            Arc::new(move |_, _, props| props.queue_flags.contains(QueueFlags::GRAPHICS)),
            Arc::new(move |device, id, _| {
                d_instance
                    .get_physical_device_surface_support(device, id as u32, surface_khr)
                    .unwrap_or(false)
            }),
        );

        let physical_device = {
            Self::iterate_physical_devices(instance.clone())?
                .map(|pdev| {
                    (
                        Self::rate_physical_device(&instance, pdev, qfi.clone()),
                        pdev,
                    )
                })
                .max_by_key(|s| s.0)
        };
        let Some(physical_device) = physical_device else {
            return Err("No physical device found.".into());
        };
        if physical_device.0 <= 0 {
            return Err("No suitable physical device found.".into());
        }
        let physical_device = physical_device.1;

        qfi.fill(&instance, physical_device);
        let queue_info = DeviceQueueCreateInfo::default()
            .queue_family_index(qfi.graphics.unwrap() as u32)
            .queue_priorities(&[0.0f32]);
        let queue_infos = vec![queue_info];

        let device_features = PhysicalDeviceFeatures::default();
        let device_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_features(&device_features);
        let device = instance.create_device(physical_device, &device_info)?;
        let queue = unsafe { device.get_device_queue(qfi.graphics.unwrap() as u32, 0) };
        Ok(Self {
            _physical_device: physical_device,
            _queue_families: qfi,
            device: Some(device),
            queue,
            instance,
        })
    }
    pub fn destroy_device(&mut self) {
        if let Some(device) = self.device.as_ref() {
            unsafe { device.destroy_device(None) };
        }
    }
}
impl Drop for DeviceManager {
    fn drop(&mut self) {
        self.destroy_device();
    }
}
