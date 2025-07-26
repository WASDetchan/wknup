use ash::{
    Device, Entry, Instance,
    khr::{self, surface},
    vk::{
        self, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures,
        PhysicalDeviceType, Queue, QueueFamilyProperties, QueueFlags, SurfaceKHR,
    },
};
use extensions::ExtensionManager;
use std::{error::Error, rc::Rc, vec::IntoIter};
use validation::ValidationLayerManager;

use crate::window::WindowManager;

pub struct VulkanManager {
    entry: Entry,
    instance: Instance,
    extension_manager: ExtensionManager,
    window_manager: WindowManager,
    device_manager: Option<DeviceManager>,
    surface: Option<vk::SurfaceKHR>,
    surface_instance: khr::surface::Instance,
}

impl VulkanManager {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let entry = Entry::linked();
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 1, 0),
            ..Default::default()
        };

        let window_manager = WindowManager::init();
        let wm_required_extensions = window_manager.get_vk_extensions();

        let mut extension_manager = ExtensionManager::init(&entry)?;
        extension_manager.add_extensions(&wm_required_extensions?)?;
        let extension_names = extension_manager.make_load_extension_list()?;

        let validation_layers = vec![String::from("VK_LAYER_KHRONOS_validation")];
        let mut validation_manager = ValidationLayerManager::init(&entry)?;
        validation_manager.add_layers(&validation_layers)?;
        let validation_layer_names = validation_manager.make_load_layer_list()?;

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&validation_layer_names)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&create_info, None)? };
        let surface_instance = khr::surface::Instance::new(&entry, &instance);

        let mut vulkan_manager = Self {
            entry,
            instance,
            extension_manager,
            window_manager,
            device_manager: None,
            surface_instance,
            surface: None,
        };

        vulkan_manager.init_surface()?;

        let device_manager = unsafe {
            DeviceManager::init(
                &vulkan_manager.instance,
                vulkan_manager.surface.unwrap().clone(),
                vulkan_manager.surface_instance.clone(),
            )?
        };

        vulkan_manager.device_manager = Some(device_manager);

        Ok(vulkan_manager)
    }

    pub fn init_surface(&mut self) -> Result<(), Box<dyn Error>> {
        self.surface = Some(
            self.window_manager
                .create_vk_surface(self.instance.handle())?,
        );
        Ok(())
    }
}
impl Drop for VulkanManager {
    fn drop(&mut self) {
        if let Some(surface_khr) = self.surface.take() {
            unsafe {
                self.surface_instance.destroy_surface(surface_khr, None);
            }
        }

        if let Some(dm) = self.device_manager.as_mut() {
            unsafe {
                dm.destroy_device();
            }
        }
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

mod extensions;

mod validation;

type Filter =
    Box<dyn Fn(&Instance, &PhysicalDevice, usize, &QueueFamilyProperties) -> bool + Clone>;
#[derive(Clone)]
struct QueueFamilyIndicies {
    graphics: Option<usize>,
    present: Option<usize>,
    graphics_filter: Filter,
    present_filter: Filter,
}

struct Queues {
    graphics: Queue,
    present: Queue,
}

impl QueueFamilyIndicies {
    fn new(graphics_filter: Filter, present_filter: Filter) -> Self {
        Self {
            graphics: None,
            present: None,
            graphics_filter,
            present_filter,
        }
    }
    unsafe fn try_queue(
        &mut self,
        instance: &Instance,
        physical_device: &PhysicalDevice,
        id: usize,
        props: &QueueFamilyProperties,
    ) {
        if (self.graphics_filter)(instance, physical_device, id, props) {
            self.graphics = Some(id);
        };
        if (self.present_filter)(instance, physical_device, id, props) {
            self.present = Some(id);
        }
    }
    unsafe fn fill(&mut self, instance: &Instance, physical_device: &PhysicalDevice) {
        unsafe { Self::iterate_physical_device_queue_families(instance, physical_device) }
            .enumerate()
            .for_each(|(id, prop)| unsafe { self.try_queue(instance, physical_device, id, &prop) });
    }
    fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.present.is_some()
    }
    unsafe fn iterate_physical_device_queue_families(
        instance: &Instance,
        physical_device: &PhysicalDevice,
    ) -> IntoIter<QueueFamilyProperties> {
        unsafe { instance.get_physical_device_queue_family_properties(physical_device.clone()) }
            .into_iter()
    }
}

struct DeviceManager {
    _physical_device: PhysicalDevice,
    _queue_families: QueueFamilyIndicies,
    device: Device,
    queue: Queue,
}
impl DeviceManager {
    unsafe fn iterate_physical_devices(
        instance: &Instance,
    ) -> Result<IntoIter<PhysicalDevice>, Box<dyn Error>> {
        Ok(unsafe { instance.enumerate_physical_devices() }?.into_iter())
    }

    unsafe fn rate_physical_device(
        instance: &Instance,
        device: &PhysicalDevice,
        mut qfi: QueueFamilyIndicies,
    ) -> u32 {
        let props = unsafe { instance.get_physical_device_properties(device.clone()) };
        let features = unsafe { instance.get_physical_device_features(device.clone()) };
        unsafe { qfi.fill(instance, device) };
        ((props.device_type == PhysicalDeviceType::DISCRETE_GPU
            || props.device_type == PhysicalDeviceType::INTEGRATED_GPU)
            && (features.geometry_shader == 1)
            && qfi.is_complete()) as u32
    }

    pub unsafe fn init(
        instance: &Instance,
        surface_khr: SurfaceKHR,
        surface_instance: surface::Instance,
    ) -> Result<Self, Box<dyn Error>> {
        let mut qfi = QueueFamilyIndicies::new(
            Box::new(|_, _, _, props| props.queue_flags.contains(QueueFlags::GRAPHICS)),
            Box::new(|_, device, id, _| unsafe {
                surface_instance
                    .get_physical_device_surface_support(device.clone(), id as u32, surface_khr)
                    .unwrap_or(false)
            }),
        );

        let physical_device = unsafe {
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

        unsafe {
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
        let device = unsafe { instance.create_device(physical_device, &device_info, None)? };
        let queue = unsafe { device.get_device_queue(qfi.graphics.unwrap() as u32, 0) };
        Ok(Self {
            _physical_device: physical_device,
            _queue_families: qfi,
            device,
            queue,
        })
    }
    pub unsafe fn destroy_device(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}
