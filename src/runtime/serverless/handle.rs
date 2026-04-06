use bytes::Bytes;
use tokio::sync::oneshot;

use crate::runtime::{
    WorkerTask,
    serverless::{
        code_store::CodeStoreError,
        trigger::{ServerlessTrigger, ServerlessTx},
    },
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
    #[must_use]
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

    /// Upload worker code.
    #[inline]
    #[must_use]
    pub async fn upload_worker(&self, name: String, code: Bytes) -> Option<CodeStoreError> {
        let (reply, recv) = oneshot::channel();
        self.trigger(ServerlessTrigger::UploadWorkerCode { name, code, reply })
            .await?;

        recv.await.ok()?
    }

    /// Trigger the serverless runtime.
    #[inline]
    #[must_use]
    pub async fn trigger(&self, trigger: ServerlessTrigger) -> Option<()> {
        self.tx.send(trigger).await.ok()
    }
}
