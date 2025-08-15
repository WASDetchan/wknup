use core::task::Waker;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::{self, JoinHandle};

use ash::vk;
use std::time::Duration;

use super::device::Device;
use super::error::fatal_vk_error;

const FENCE_POLL_PERIOD: Duration = Duration::from_micros(100000);

enum FenceState {
    Ready(vk::Fence),
    Waiting(JoinHandle<vk::Fence>),
}

use FenceState::{Ready, Waiting};

impl FenceState {
    fn start_wait(&mut self, device: Arc<Device>, waker: Waker) {
        let Ready(fence) = *self else {
            panic!("Tried starting waiting for a fence that is already being waited for!");
        };
        *self = Waiting(thread::spawn(move || {
            loop {
                let code = unsafe {
                    device.raw_handle().wait_for_fences(
                        &[fence],
                        true,
                        FENCE_POLL_PERIOD.as_nanos().try_into().unwrap(),
                    )
                };
                if check_shutdown() {
                    break;
                }
                let Err(error) = code else {
                    break;
                };
                if error != vk::Result::TIMEOUT {
                    fatal_vk_error("failed to wait_for_fences", error);
                }
            }
            waker.wake();
            fence
        }))
    }

    fn wait(&mut self) {
        if let Ready(_) = *self {
            return;
        }
        let s = mem::replace(self, Ready(vk::Fence::null()));
        let Waiting(handle) = s else {
            unreachable!();
        };
        *self = Ready(handle.join().unwrap());
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        match self.fence {
            Ready(fence) => unsafe {
                self.device.raw_handle().destroy_fence(fence, None);
            },
            _ => {
                panic!("FenceState cannot be dropped while being waited!");
            }
        }
    }
}

pub struct Fence {
    device: Arc<Device>,
    fence: FenceState,
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
            fence: FenceState::Ready(fence),
            #[cfg(debug_assertions)]
            name: String::new(),
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            let Ready(fence) = self.fence else {
                panic!("Fence cannot be reset while being waited for!");
            };
            self.device
                .raw_handle()
                .reset_fences(&[fence])
                .unwrap_or_else(|error| fatal_vk_error("failed to reset fence", error));
        }
    }
    pub(in crate::vk) unsafe fn raw_handle(&self) -> vk::Fence {
        match self.fence {
            Ready(fence) => fence,
            _ => {
                panic!("vk::Fence cannot be retrieved while being waited!");
            }
        }
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
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Ready(fence) = self.fence else {
            self.fence.wait();
            if check_shutdown() {
                self.polled_after_shutdown();
            }

            return Poll::Ready(());
        };
        match unsafe { self.device.raw_handle().get_fence_status(fence) } {
            Ok(true) => Poll::Ready(()),
            Ok(false) => {
                let device_clone = Arc::clone(&self.device);
                self.fence.start_wait(device_clone, cx.waker().clone());
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

// fn spawn_poller(device: Arc<Device>, fence: vk::Fence, waker: Waker) {
//     tokio::spawn(async move {
//         loop {
//             if check_shutdown()
//                 || unsafe { device.raw_handle().get_fence_status(fence) } != Ok(false)
//             {
//                 waker.wake();
//                 break;
//             } else {
//                 tokio::time::sleep(FENCE_POLL_PERIOD).await;
//             }
//         }
//     });
// }

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
