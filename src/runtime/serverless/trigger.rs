use tokio::sync::{mpsc, oneshot};

use crate::runtime::{PodTrigger, WorkerTask};

#[derive(Debug)]
pub enum ServerlessTrigger {
    CreateWorker {
        task: WorkerTask,
        reply: oneshot::Sender<Option<(usize, usize)>>,
    },

    SetUniversalWorkerName {
        name: String,
        locator: (usize, usize),
    },

    RemoveUniversalWorkerName {
        name: String,
    },

    ToPod {
        id: usize,
        trigger: PodTrigger,
    },
}

pub(super) type ServerlessTx = mpsc::Sender<ServerlessTrigger>;
pub(super) type ServerlessRx = mpsc::Receiver<ServerlessTrigger>;
