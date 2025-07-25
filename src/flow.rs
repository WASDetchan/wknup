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
