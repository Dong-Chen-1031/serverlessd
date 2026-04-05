use tokio::sync::mpsc;

use crate::runtime::{
    Pod, WorkerTask,
    worker::{
        task::create_cancel_safe_task,
        trigger::{WorkerTrigger, WorkerTx},
    },
};

/// A handle to the serverless worker.
#[derive(Debug)]
#[repr(transparent)]
pub struct WorkerHandle {
    tx: WorkerTx,
}

impl WorkerHandle {
    #[inline]
    pub fn start(pod: &Pod, task: WorkerTask) -> Self {
        let (tx, rx) = mpsc::channel::<WorkerTrigger>(64);

        let monitor = pod.monitor.clone();

        pod.tasks
            .spawn_local(create_cancel_safe_task(task, tx.clone(), rx, monitor));
        Self { tx }
    }

    /// Trigger.
    ///
    /// Returns `false` if the channel is closed.
    #[inline(always)]
    #[must_use]
    pub async fn trigger(&self, trigger: WorkerTrigger) -> bool {
        self.tx.send(trigger).await.is_ok()
    }
}
