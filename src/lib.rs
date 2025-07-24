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

mod window {
    use crate::mpsc;
    use winit::{
        self,
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{self, EventLoop},
        window::{Window, WindowAttributes},
    };

    struct WindowManager {
        window: Option<Window>,
        attrubutes: WindowAttributes,
        tx: mpsc::UnboundedSender<WindowEvent>,
    }

    impl ApplicationHandler for WindowManager {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            self.window = Some(event_loop.create_window(self.attrubutes));
        }
        fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            self.window = None;
        }
        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
            self.tx.send(event);
        }
    }
    fn run() {
        let event_loop = EventLoop::new();
    }
}

pub mod mpsc {
    pub use tokio::sync::mpsc::{Sender, UnboundedSender, channel, unbounded_channel};
    pub enum Receiver<V> {
        Bounded(mpsc::Receiver<V>),
        Unbounded(mpsc::UnboundedReceiver<V>),
    }
    impl Receiver<V> {
        pub async fn recv(&mut self) {
            match self {
                Self::Bounded(r) => r.recv(),
                Self::Unbounded(r) => r.recv(),
            }
        }
    }
}
