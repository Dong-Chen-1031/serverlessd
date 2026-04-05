use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
#[allow(unused)]
pub enum WorkerTrigger {
    Http { reply: oneshot::Sender<String> },
    Halt { token: oneshot::Sender<()> },
}

pub type WorkerTx = mpsc::Sender<WorkerTrigger>;
pub(super) type WorkerRx = mpsc::Receiver<WorkerTrigger>;
