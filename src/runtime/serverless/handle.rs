use tokio::sync::oneshot;

use crate::runtime::{
    WorkerTask,
    serverless::trigger::{ServerlessTrigger, ServerlessTx},
};

#[repr(transparent)]
pub struct ServerlessHandle {
    tx: ServerlessTx,
}

impl ServerlessHandle {
    #[inline(always)]
    pub(super) fn new(tx: ServerlessTx) -> Self {
        Self { tx }
    }

    /// Notifies the serverless runtime to create a worker.
    pub async fn create_worker(&self, task: WorkerTask) -> Option<(usize, usize)> {
        let (reply, receive) = oneshot::channel();
        self.tx
            .send(ServerlessTrigger::CreateWorker { task, reply })
            .await
            .ok()?;

        let Ok(result) = receive.await else {
            return None;
        };

        result
    }

    /// Trigger the serverless runtime.
    #[inline]
    pub async fn trigger(&self, trigger: ServerlessTrigger) -> Option<()> {
        self.tx.send(trigger).await.ok()
    }
}
