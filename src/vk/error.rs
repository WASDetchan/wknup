use ash::vk;

#[derive(strum::FromRepr, strum::Display)]
#[repr(i32)]
enum VulkanResult {
    #[doc = "Command completed successfully"]
    SUCCESS = 0,
    #[doc = "A fence or query has not yet completed"]
    NOT_READY = 1,
    #[doc = "A wait operation has not completed in the specified time"]
    TIMEOUT = 2,
    #[doc = "An event is signaled"]
    EVENT_SET = 3,
    #[doc = "An event is unsignaled"]
    EVENT_RESET = 4,
    #[doc = "A return array was too small for the result"]
    INCOMPLETE = 5,
    #[doc = "A host memory allocation has failed"]
    ERROR_OUT_OF_HOST_MEMORY = -1,
    #[doc = "A device memory allocation has failed"]
    ERROR_OUT_OF_DEVICE_MEMORY = -2,
    #[doc = "Initialization of an object has failed"]
    ERROR_INITIALIZATION_FAILED = -3,
    #[doc = "The logical device has been lost. See <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#devsandqueues-lost-device>"]
    ERROR_DEVICE_LOST = -4,
    #[doc = "Mapping of a memory object has failed"]
    ERROR_MEMORY_MAP_FAILED = -5,
    #[doc = "Layer specified does not exist"]
    ERROR_LAYER_NOT_PRESENT = -6,
    #[doc = "Extension specified does not exist"]
    ERROR_EXTENSION_NOT_PRESENT = -7,
    #[doc = "Requested feature is not available on this device"]
    ERROR_FEATURE_NOT_PRESENT = -8,
    #[doc = "Unable to find a Vulkan driver"]
    ERROR_INCOMPATIBLE_DRIVER = -9,
    #[doc = "Too many objects of the type have already been created"]
    ERROR_TOO_MANY_OBJECTS = -10,
    #[doc = "Requested format is not supported on this device"]
    ERROR_FORMAT_NOT_SUPPORTED = -11,
    #[doc = "A requested pool allocation has failed due to fragmentation of the pool's memory"]
    ERROR_FRAGMENTED_POOL = -12,
    #[doc = "An unknown error has occurred, due to an implementation or application bug"]
    ERROR_UNKNOWN = -13,
}

impl VulkanResult {
    fn doc(&self) -> &'static str {
        match self {
            Self::SUCCESS => "Command completed successfully",
            Self::NOT_READY => "A fence or query has not yet completed",
            Self::TIMEOUT => "A wait operation has not completed in the specified time",
            Self::EVENT_SET => "An event is signaled",
            Self::EVENT_RESET => "An event is unsignaled",
            Self::INCOMPLETE => "A return array was too small for the result",
            Self::ERROR_OUT_OF_HOST_MEMORY => "A host memory allocation has failed",
            Self::ERROR_OUT_OF_DEVICE_MEMORY => "A device memory allocation has failed",
            Self::ERROR_INITIALIZATION_FAILED => "Initialization of an object has failed",
            Self::ERROR_DEVICE_LOST => {
                "The logical device has been lost. See <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#devsandqueues-lost-device>"
            }
            Self::ERROR_MEMORY_MAP_FAILED => "Mapping of a memory object has failed",
            Self::ERROR_LAYER_NOT_PRESENT => "Layer specified does not exist",
            Self::ERROR_EXTENSION_NOT_PRESENT => "Extension specified does not exist",
            Self::ERROR_FEATURE_NOT_PRESENT => "Requested feature is not available on this device",
            Self::ERROR_INCOMPATIBLE_DRIVER => "Unable to find a Vulkan driver",
            Self::ERROR_TOO_MANY_OBJECTS => {
                "Too many objects of the type have already been created"
            }
            Self::ERROR_FORMAT_NOT_SUPPORTED => "Requested format is not supported on this device",
            Self::ERROR_FRAGMENTED_POOL => {
                "A requested pool allocation has failed due to fragmentation of the pool's memory"
            }
            Self::ERROR_UNKNOWN => {
                "An unknown error has occurred, due to an implementation or application bug"
            }
        }
    }
}

impl From<vk::Result> for VulkanResult {
    fn from(value: vk::Result) -> Self {
        Self::from_repr(value.as_raw()).expect("all VkResult cases are covered")
    }
}

pub fn fatal_vk_error<T: Into<VulkanResult>>(msg: &str, error: T) -> ! {
    let e = error.into();
    panic!("fatal: {}: {} ({})", msg, e, e.doc());
}
