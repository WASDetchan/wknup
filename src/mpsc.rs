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
