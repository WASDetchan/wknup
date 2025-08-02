mod device_extensions;

use std::{error::Error, ffi::CStr, sync::Arc, vec::IntoIter};

use ash::{
    Device,
    vk::{
        DeviceCreateInfo, DeviceQueueCreateInfo, ExtensionProperties, PhysicalDevice,
        PhysicalDeviceFeatures, PhysicalDeviceProperties, PhysicalDeviceType, Queue,
        QueueFamilyProperties, QueueFlags, SurfaceKHR,
    },
};
use device_extensions::DeviceExtensionManager;

use super::instance::InstanceManager;

const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 1] = [c"VK_KHR_swapchain"];

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
    physical_device: Option<PhysicalDevice>,
    queue_families: QueueFamilyIndicies,
    instance: Arc<InstanceManager>,
    device: Option<Device>,
    queues: Option<Queues>,
    surface: SurfaceKHR,
}
impl DeviceManager {
    fn new(instance: Arc<InstanceManager>, surface: SurfaceKHR) -> Self {
        Self {
            physical_device: None,
            queue_families: Self::make_qfi(instance.clone(), surface),
            instance,
            device: None,
            queues: None,
            surface,
        }
    }
    fn iterate_physical_devices(
        instance: &Arc<InstanceManager>,
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

    fn init_physical_device(&mut self) -> Result<(), Box<dyn Error>> {
        let physical_device = Self::iterate_physical_devices(&self.instance)?
            .map(|pdev| {
                (
                    Self::rate_physical_device(&self.instance, pdev, self.queue_families.clone()),
                    pdev,
                )
            })
            .max_by_key(|s| s.0);
        let Some(physical_device) = physical_device else {
            return Err("No physical device found.".into());
        };
        if physical_device.0 <= 0 {
            return Err("No suitable physical device found.".into());
        }
        self.physical_device = Some(physical_device.1);
        self.queue_families
            .fill(&self.instance, self.physical_device.unwrap());
        Ok(())
    }

    fn make_qfi(instance: Arc<InstanceManager>, surface_khr: SurfaceKHR) -> QueueFamilyIndicies {
        QueueFamilyIndicies::new(
            Arc::new(move |_device, _id, props| props.queue_flags.contains(QueueFlags::GRAPHICS)),
            Arc::new(move |device, id, _props| {
                instance
                    .get_physical_device_surface_support(device, id as u32, surface_khr)
                    .unwrap_or(false)
            }),
        )
    }

    fn init_device(&mut self) -> Result<(), Box<dyn Error>> {
        let qfi = &self.queue_families;

        let graphic_present_match = qfi.graphics.unwrap() == qfi.present.unwrap();

        let graphic_info = DeviceQueueCreateInfo::default()
            .queue_family_index(qfi.graphics.unwrap() as u32)
            .queue_priorities(&[0.0f32]);
        let present_info = DeviceQueueCreateInfo::default()
            .queue_family_index(qfi.present.unwrap() as u32)
            .queue_priorities(&[0.0f32]);

        let queue_infos = if graphic_present_match {
            vec![graphic_info]
        } else {
            vec![graphic_info, present_info]
        };

        let device_features = PhysicalDeviceFeatures::default();

        let mut device_extension_manager = DeviceExtensionManager::init(self)?;

        device_extension_manager.add_extensions(&REQUIRED_DEVICE_EXTENSIONS)?;
        let ext_names = device_extension_manager.list_names();

        let device_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&ext_names);
        let Some(physical_device) = self.physical_device else {
            return Err("cannot init device before physical device is initialized".into());
        };
        self.device = Some(self.instance.create_device(physical_device, &device_info)?);
        Ok(())
    }

    fn init_queues(&mut self) {
        let qfi = &self.queue_families;
        let device = self.device.as_ref().unwrap();

        let graphic_queue = unsafe { device.get_device_queue(qfi.graphics.unwrap() as u32, 0) };
        let present_queue = unsafe { device.get_device_queue(qfi.present.unwrap() as u32, 0) };

        let queues = Queues {
            graphics: graphic_queue,
            present: present_queue,
        };
        self.queues = Some(queues);
    }

    fn query_swapchain_support(&self, device: PhysicalDevice) -> Result<(), Box<dyn Error>> {
        unsafe {
            let (surface_capabilities, surface_formats, present_modes) = self
                .instance
                .get_physical_device_surface_info(device, self.surface)?;
        }
        Ok(())
    }

    pub fn init(
        instance: Arc<InstanceManager>,
        surface_khr: SurfaceKHR,
    ) -> Result<Self, Box<dyn Error>> {
        let mut device_manager = Self::new(instance, surface_khr);
        device_manager.init_physical_device()?;
        device_manager.init_device()?;
        device_manager.init_queues();

        Ok(device_manager)
    }
    pub fn destroy_device(&mut self) {
        if let Some(device) = self.device.as_ref() {
            unsafe { device.destroy_device(None) };
        }
    }
    pub fn enumerate_device_extension_properties(
        &self,
    ) -> Result<Vec<ExtensionProperties>, Box<dyn Error>> {
        let Some(physical_device) = self.physical_device else {
            return Err("cannot enumerate_device_extension_properties before physical device is initialized".into());
        };
        unsafe {
            self.instance
                .enumerate_device_extension_properties(physical_device)
        }
    }
}
impl Drop for DeviceManager {
    fn drop(&mut self) {
        self.destroy_device();
    }
}
