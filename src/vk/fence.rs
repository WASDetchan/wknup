use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use ash::vk;
use tokio::time::Duration;

use super::device::Device;
use super::error::fatal_vk_error;

const FENCE_POLL_PERIOD: Duration = Duration::from_micros(1000);

pub struct Fence {
    device: Arc<Device>,
    fence: vk::Fence,
    #[cfg(debug_assertions)]
    name: String,
}

impl Fence {
    pub fn new(device: Arc<Device>) -> Self {
        let create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let fence = unsafe {
            device
                .raw_handle()
                .create_fence(&create_info, None)
                .unwrap_or_else(|error| fatal_vk_error("failed to create_fence", error))
        };

        Self {
            device,
            fence,
            #[cfg(debug_assertions)]
            name: String::new(),
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            self.device
                .raw_handle()
                .reset_fences(&[self.fence])
                .unwrap_or_else(|error| fatal_vk_error("failed to reset fence", error));
        }
    }
    pub(in crate::vk) unsafe fn raw_handle(&self) -> vk::Fence {
        self.fence
    }

    #[cfg(debug_assertions)]
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    #[cfg(debug_assertions)]
    pub fn polled_after_shutdown(&self) {
        eprintln!("Fence \"{}\" was polled after shutdown!", self.name);
    }

    #[cfg(not(debug_assertions))]
    pub const fn set_name(&mut self, _: &str) {}

    #[cfg(not(debug_assertions))]
    pub fn polled_after_shutdown(&self) {}
}

impl Future for Fence {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if check_shutdown() {
            self.polled_after_shutdown();
            return Poll::Ready(());
        }
        match unsafe { self.device.raw_handle().get_fence_status(self.fence) } {
            Ok(true) => Poll::Ready(()),
            Ok(false) => {
                spawn_poller(self.device.clone(), self.fence, cx.waker().clone());
                Poll::Pending
            }
            Err(error) => fatal_vk_error("failed to get_fence_status", error),
        }
    }
}

#[cfg(debug_assertions)]
fn check_shutdown() -> bool {
    use std::sync::atomic::Ordering::Relaxed;

    crate::FENCE_SHUTDOWN.load(Relaxed)
}

#[cfg(not(debug_assertions))]
const fn check_shutdown() -> bool {
    false
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.raw_handle().destroy_fence(self.fence, None);
        }
    }
}

fn spawn_poller(device: Arc<Device>, fence: vk::Fence, waker: core::task::Waker) {
    tokio::spawn(async move {
        loop {
            if check_shutdown()
                || unsafe { device.raw_handle().get_fence_status(fence) } != Ok(false)
            {
                waker.wake();
                break;
            } else {
                tokio::time::sleep(FENCE_POLL_PERIOD).await;
            }
        }
    });
}

// let waker = cx.waker().clone();
//                 let fence = self.fence;
//                 let device = self.device.clone();
//                 std::thread::spawn(move || unsafe {
//                     loop {
//                         let code = device.raw_handle().wait_for_fences(
//                             &[fence],
//                             true,
//                             FENCE_POLL_PERIOD.as_nanos().try_into().unwrap(),
//                         );
//
//                         println!("{:?}", code);
//
//                         match code {
//                             Ok(()) => {
//                                 break;
//                             }
//                             Err(vk::Result::TIMEOUT) => {
//                                 if check_shutdown() {
//                                     break;
//                                 }
//                             }
//                             Err(error) => fatal_vk_error("failed to wait_for_fences", error),
//                         }
//                     }
//                     waker.wake();
//                     println!("drop");
//                 });
//
