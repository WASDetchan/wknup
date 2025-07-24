mod base {}

mod flow {

    use tokio::sync::mpsc::{self, Receiver, Sender};
    pub trait Transformer<F, T> {
        fn set_tx(&mut self, tx: Sender<T>);
        fn set_rx(&mut self, rx: Receiver<F>);
        async fn run();
    }
    pub fn connect<V, T, R, V1, V2>(transmitter: &mut T, reciever: &mut R, buffer: usize)
    where
        V: Send,
        T: Transformer<V1, V>,
        R: Transformer<V, V2>,
    {
        let (tx, mut rx) = mpsc::channel::<V>(buffer);
        transmitter.set_tx(tx);
        reciever.set_rx(rx);
    }
}

pub mod mpsc {
    pub use tokio::sync::mpsc::{Sender, UnboundedSender, channel, unbounded_channel};
    pub enum Receiver<V> {
        Bounded(tokio::sync::mpsc::Receiver<V>),
        Unbounded(tokio::sync::mpsc::UnboundedReceiver<V>),
    }
    impl<V> Receiver<V> {
        pub async fn recv(
            &mut self,
        ) -> Box<dyn std::future::Future<Output = std::option::Option<V>> + '_> {
            match self {
                Self::Bounded(r) => Box::new(r.recv()),
                Self::Unbounded(r) => Box::new(r.recv()),
            }
        }
    }
}

pub mod vk {
    use ash::{Entry, Instance, vk};
    use std::error::Error;

    struct VulkanManager {
        entry: Entry,
        instance: Instance,
    }

    impl VulkanManager {
        fn init() -> Result<Self, Box<dyn Error>> {
            let entry = Entry::linked();
            let app_info = vk::ApplicationInfo {
                api_version: vk::make_api_version(0, 1, 1, 0),
                ..Default::default()
            };
            let create_info = vk::InstanceCreateInfo {
                p_application_info: &app_info,
                ..Default::default()
            };
            let instance = unsafe { entry.create_instance(&create_info, None)? };
            Ok(Self { entry, instance })
        }
    }
    impl Drop for VulkanManager {
        fn drop(&mut self) {
            unsafe {
                self.instance.destroy_instance(None);
            }
        }
    }
}

pub mod window {
    use std::error::Error;

    use sdl3::{
        self, Sdl, VideoSubsystem,
        video::{VkInstance, VkSurfaceKHR, Window},
    };

    struct WindowManager {
        sdl_context: Sdl,
        video_subsystem: VideoSubsystem,
        window: Window,
    }

    impl WindowManager {
        fn init() -> Self {
            let sdl_context = sdl3::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let window = video_subsystem
                .window("Test window", 800, 600)
                .position_centered()
                .vulkan()
                .build()
                .unwrap();

            Self {
                sdl_context,
                video_subsystem,
                window,
            }
        }
        fn create_vk_surface(&self, instance: VkInstance) -> Result<VkSurfaceKHR, Box<dyn Error>> {
            Ok(self.window.vulkan_create_surface(instance)?)
        }
        fn get_vk_extensions(&self) -> Result<Vec<String>, Box<dyn Error>> {
            Ok(self.window.vulkan_instance_extensions()?)
        }
    }
}
