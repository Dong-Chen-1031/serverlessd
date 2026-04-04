use std::{pin::Pin, task, time::Duration};

use tokio::sync::mpsc;
use v8::IsolateHandle;

pub struct Monitor {
    // SAFETY: we don't need to get a stable memory address
    // # of pod workers will not increase
    workers: Vec<Option<MonitoredWorker>>,
}

impl Monitor {
    /// Creates a wall time monitor manager.
    ///
    /// To keep things simple, the monitor creates an uninitialized
    /// [`Vec`] of memory with the capacity of `n_workers` assigned.
    /// This is slightly different from a `Pod`, which initializes
    /// only when needed, while allocating the same amount of memory.
    pub fn new(n_workers: usize) -> Self {
        let mut workers = Vec::with_capacity(n_workers);
        for _ in 0..n_workers {
            workers.push(None);
        }

        Self { workers }
    }

    /// Sets the monitor for `worker_id`.
    ///
    /// # Safety
    /// Worker of ID `worker_id` must exist.
    pub fn put(
        &mut self,
        isolate_handle: IsolateHandle,
        worker_id: usize,
    ) -> MonitoredFutureFactory {
        let spot = unsafe { self.workers.get_mut(worker_id).unwrap_unchecked() };
        let (tx, rx) = mpsc::channel(1);

        let mw = MonitoredWorker::new(isolate_handle, rx);
        spot.replace(mw);

        MonitoredFutureFactory(tx)
    }
}

pub struct MonitoredWorker {
    isolate: IsolateHandle,
    rx: mpsc::Receiver<()>,
}

impl MonitoredWorker {
    #[inline(always)]
    pub fn new(isolate: IsolateHandle, rx: mpsc::Receiver<()>) -> Self {
        Self { isolate, rx }
    }
}

#[repr(transparent)]
pub struct MonitoredFutureFactory(mpsc::Sender<()>);

impl MonitoredFutureFactory {
    #[inline(always)]
    pub fn monitored<F: Future>(&self, fut: F) -> MonitoredFuture<F> {
        MonitoredFuture {
            inner: fut,
            tx: self.0.clone(),
        }
    }
}

pub struct MonitoredFuture<F> {
    inner: F,
    tx: mpsc::Sender<()>,
}

impl<F: Future> Future for MonitoredFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(&mut this.inner) };

        this.tx.send(());
        let result = inner.poll(cx);
        this.tx.send(());

        result
    }
}

#[tracing::instrument(skip_all)]
async fn monitor_worker(mut mw: MonitoredWorker) {
    while let Some(()) = mw.rx.recv().await {
        tokio::select! {
            // we still like the user at some point
            biased;

            _ = mw.rx.recv() => {}
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                // time's up bitch
                if !mw.isolate.terminate_execution() {
                    tracing::error!("failed to terminate isolate when 10ms time's up");
                }
            }
        };
    }
}
