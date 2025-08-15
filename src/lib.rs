#[cfg(debug_assertions)]
static FENCE_SHUTDOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// makes every fence ready. every fence that is polled after prints an error
#[cfg(debug_assertions)]
pub fn fence_shutdown() {
    log::warn!("Shutting down fences!");
    FENCE_SHUTDOWN.store(true, std::sync::atomic::Ordering::Relaxed);
}

mod base {}

// mod flow;

// pub mod mpsc;

pub mod vk;

pub mod window;
