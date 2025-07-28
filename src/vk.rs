use ash::{
    vk::{self, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceProperties, PhysicalDeviceType, Queue, QueueFamilyProperties, SurfaceKHR}, Device, Entry
};
use core::fmt;
use instance::InstanceManager;
use std::{error::Error, sync::Arc, vec::IntoIter};

use crate::window::WindowManager;

pub mod instance;

#[derive(Debug)]
enum VulkanInitStage {
    Entry,
    WindowManager,
    InstanceManager,
    Surface,
}

impl fmt::Display for VulkanInitStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                VulkanInitStage::Entry => "Entry",
                VulkanInitStage::WindowManager => "WindowManager",
                VulkanInitStage::InstanceManager => "InstanceManager",
                VulkanInitStage::Surface => "Surface",
            }
        )
    }
}
#[derive(Debug)]
struct VulkanInitOrderError {
    attempted_stage: VulkanInitStage,
    requiered_stage: VulkanInitStage,
}
impl fmt::Display for VulkanInitOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unable to initialize {} before {} is initalized",
            self.attempted_stage, self.requiered_stage
        )
    }
}

impl Error for VulkanInitOrderError {}

#[derive(Default)]
pub struct VulkanManager {
    entry: Option<Arc<Entry>>,
    instance_manager: Option<Arc<InstanceManager>>,
    window_manager: Option<WindowManager>,
    // device_manager: Option<DeviceManager>,
}

impl VulkanManager {
    pub fn new() -> Self {
        Self::default()
    }
    fn init_entry(&mut self) {
        self.entry = Some(Arc::new(Entry::linked()));
    }
    fn init_window_manager(&mut self) {
        self.window_manager = Some(WindowManager::init());
    }
    fn init_instance(&mut self) -> Result<(), Box<dyn Error>> {
        let Some(entry) = self.entry.clone() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::InstanceManager,
                requiered_stage: VulkanInitStage::Entry,
            }));
        };
        let Some(window_manager) = self.window_manager.as_ref() else {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::InstanceManager,
                requiered_stage: VulkanInitStage::WindowManager,
            }));
        };

        let wm_required_extensions = window_manager.get_vk_extensions()?;

        let mut instance_manager = InstanceManager::init(entry)?
            .extensions(wm_required_extensions)
            .validation_layers(vec![String::from("VK_LAYER_KHRONOS_validation")])
            .application_props(String::from("WKNUP"), 1)
            .api_version(vk::make_api_version(0, 1, 1, 0));
        instance_manager.init_instance()?;

        self.instance_manager = Some(Arc::new(instance_manager));

        Ok(())
    }

    fn init_surface(&mut self) -> Result<(), Box<dyn Error>> {
        if self.instance_manager.is_none() {
            return Err(Box::new(VulkanInitOrderError {
                attempted_stage: VulkanInitStage::Surface,
                requiered_stage: VulkanInitStage::InstanceManager,
            }));
        };
        self.window_manager
            .as_mut()
            .expect("window_manager is always initialized at this point")
            .init_surface(Arc::clone(self.instance_manager.as_ref().unwrap()))?;
        Ok(())
    }
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let mut vulkan_manager = Self::default();
        vulkan_manager.init_entry();
        vulkan_manager.init_window_manager();
        vulkan_manager.init_instance()?;
        vulkan_manager.init_surface()?;
        Ok(vulkan_manager)
    }

}

mod extensions;

mod validation;


#[derive(Clone)]
struct QueueFamilyIndicies {
    graphics: Option<usize>,
    present: Option<usize>,
    graphics_filter: &impl Fn(&PhysicalDevice, usize, &QueueFamilyProperties) -> bool,
    present_filter: &impl Fn(&PhysicalDevice, usize, &QueueFamilyProperties) -> bool,
}

struct Queues {
    graphics: Queue,
    present: Queue,
}

impl QueueFamilyIndicies {
    fn new(graphics_filter: &impl Fn(&PhysicalDevice, usize, &QueueFamilyProperties) -> bool, present_filter: &impl Fn(&PhysicalDevice, usize, &QueueFamilyProperties) -> bool) -> Self {
        Self {
            graphics: None,
            present: None,
            graphics_filter,
            present_filter,
        }
    }
    unsafe fn try_queue(
        &mut self,
        physical_device: &PhysicalDevice,
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
    unsafe fn fill(&mut self, instance: Arc<InstanceManager>, physical_device: &PhysicalDevice) {
        unsafe { Self::iterate_physical_device_queue_families(instance, physical_device) }
            .enumerate()
            .for_each(|(id, prop)| unsafe { self.try_queue(physical_device, id, &prop) });
    }
    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
    unsafe fn iterate_physical_device_queue_families(
        instance: Arc<InstanceManager>,
        physical_device: &PhysicalDevice,
    ) -> IntoIter<QueueFamilyProperties> {
        unsafe { instance.get_physical_device_queue_family_properties(physical_device.clone()) }
            .into_iter()
    }
}

struct PhysicalDeviceInfo {
    properties: PhysicalDeviceProperties,
    features: PhysicalDeviceFeatures,
}

struct DeviceManager {
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
        Ok( instance.enumerate_physical_devices() ?.into_iter())
    }

    fn rate_physical_device(
        instance: Arc<InstanceManager>,
        device: &PhysicalDevice,
        mut qfi: QueueFamilyIndicies,
    ) -> u32 {
        let info = instance.get_physical_device_info(device.clone())?;
        let props =  info.properties;
        let features = info.features;
        { qfi.fill(instance, device) };
        ((props.device_type == PhysicalDeviceType::DISCRETE_GPU
            || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
            && (features.geometry_shader == 1)
            && qfi.is_complete()) as u32
    }

    pub fn init(
        instance: Arc<InstanceManager>,
        surface_khr: SurfaceKHR,
    ) -> Result<Self, Box<dyn Error>> {
        let mut qfi = QueueFamilyIndicies::new(
            Box::new(|_, _, _, props| props.queue_flags.contains(QueueFlags::GRAPHICS)),
            Box::new(|_, device, id, _| {
                surface_instance
                    .get_physical_device_surface_support(device.clone(), id as u32, surface_khr)
                    .unwrap_or(false)
            }),
        );

        let physical_device = {
            Self::iterate_physical_devices(instance)?
                .map(|pdev| {
                    (
                        Self::rate_physical_device(instance, &pdev, qfi.clone()),
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

        {
            qfi.fill(instance, &physical_device);
        }
        let queue_info = DeviceQueueCreateInfo::default()
            .queue_family_index(qfi.graphics.unwrap() as u32)
            .queue_priorities(&[0.0f32]);
        let queue_infos = vec![queue_info];

        let device_features = PhysicalDeviceFeatures::default();
        let device_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_features(&device_features);
        let device = { instance.create_device(physical_device, &device_info, None)? };
        let queue = { device.get_device_queue(qfi.graphics.unwrap() as u32, 0) };
        Ok(Self {
            _physical_device: physical_device,
            _queue_families: qfi,
            device,
            queue,
        })
    }
    pub fn destroy_device(&mut self) {
        if let Some(device) = self.device.as_ref() {

        unsafe { device.destroy_device(None) };
        }
    }
}
