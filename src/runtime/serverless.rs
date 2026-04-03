use std::sync::Arc;

use tokio::{
    sync::{RwLock, mpsc, oneshot},
    task::{self, JoinHandle},
};
use v8::{Platform, SharedRef};

use crate::runtime::{Pod, WorkerTask, pod::PodTrigger};

#[derive(Debug)]
pub enum ServerlessTrigger {
    CreateWorker {
        task: WorkerTask,
        reply: oneshot::Sender<Option<(usize, usize)>>,
    },
}

type ServerlessTx = mpsc::Sender<ServerlessTrigger>;
type ServerlessRx = mpsc::Receiver<ServerlessTrigger>;

/// The serverless runtime.
///
/// Example:
/// ```rs
/// let serverless = Serverless::start(
///     10, // the number of threads you need
///     10, // the number of workers per thread
/// )
/// ```
pub struct Serverless {
    // why the fuck is this super fucking big???
    // like, fucking 16 bytes
    // or whatever, if you're happy with it
    platform: SharedRef<Platform>,
    pods: Vec<Arc<RwLock<Pod>>>,

    n_threads: usize,
    n_workers: usize,
}

impl Serverless {
    /// Create a serverless runtime.
    pub fn new(n_threads: usize, n_workers: usize) -> Self {
        // we gotta initialize the platform first
        let platform = {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform.clone());
            v8::V8::initialize();

            platform
        };

        let pods = Vec::with_capacity(n_threads);

        Self {
            platform,
            pods,
            n_threads,
            n_workers,
        }
    }

    /// Create a serverless runtime for one worker only.
    #[inline]
    pub fn new_one() -> Self {
        Self::new(1, 1)
    }

    /// Starts the serverless runtime.
    #[inline]
    #[must_use]
    pub fn start(self) -> (ServerlessHandle, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(512);
        let handle = task::spawn(serverless_task(self, rx));

        (ServerlessHandle::new(tx), handle)
    }

    /// Get the platform from [`v8`].
    #[inline(always)]
    pub fn get_platform(&self) -> SharedRef<Platform> {
        self.platform.clone()
    }

    #[inline]
    async fn find_vancancy(&self) -> Option<usize> {
        for (idx, pod) in self.pods.iter().enumerate() {
            let pod = pod.read().await;
            if pod.has_vacancy() {
                return Some(idx);
            }
        }
        None
    }

    /// Stop all pods.
    async fn halt(&mut self) {
        for pod in self.pods.drain(..) {
            let mut pod = pod.write().await;
            if !pod.halt().await {
                tracing::error!("failed to halt");
            }
        }
    }

    /// Stop a pod.
    async fn halt_pod(&mut self, id: usize) -> bool {
        if let Some(pod) = self.pods.get_mut(id) {
            let mut pod = pod.write().await;
            pod.halt().await
        } else {
            false
        }
    }

    /// Finds vacancies from pods, then create a worker
    /// within the pod, eventually returning `Some()` tuple containing:
    ///
    /// `(pod_id: usize, pod_worker_id: usize)`
    ///
    /// Under one of these conditions, `None` is returned:
    /// - No vacancies available
    /// - Failed to trigger pod
    /// - Failed to receive worker id under the designated pod
    #[must_use]
    async fn create_worker(&self, task: WorkerTask) -> Option<(usize, usize)> {
        let pod_id = self.find_vancancy().await?;
        let pod = unsafe { self.pods.get(pod_id).unwrap_unchecked() };

        let receive = {
            // NOTE:
            // DO NOT remove this block!
            // if you do, we're prone to dead locks.
            //
            // essentially, we trigger it, and immediately say fuh nawh,
            // you can have the handle
            // since the pod wants to create a worker within
            let pod = pod.read().await;
            let (reply, receive) = oneshot::channel::<usize>();
            if !pod.trigger(PodTrigger::CreateWorker { task, reply }).await {
                return None;
            }
            receive
        };

        let pod_worker_id = receive.await.ok()?;
        Some((pod_id, pod_worker_id))
    }
}

#[repr(transparent)]
pub struct ServerlessHandle {
    tx: ServerlessTx,
}

impl ServerlessHandle {
    #[inline(always)]
    fn new(tx: ServerlessTx) -> Self {
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
}

async fn serverless_task(mut serverless: Serverless, mut rx: ServerlessRx) {
    // now, we gotta start those threads
    // i know, this might be a bit not so memory efficient
    let mut handles = Vec::with_capacity(serverless.n_threads);
    for _ in 0..serverless.n_threads {
        let (pod, handle) = Pod::start(serverless.n_workers);
        serverless.pods.push(pod);
        handles.push(handle);
    }

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                tracing::info!("sent halt");
                serverless.halt().await;
                break;
            },

            result = rx.recv() => {
                match result {
                    Some(trigger) => {
                        match trigger {
                            ServerlessTrigger::CreateWorker { task, reply } => {
                                reply.send(serverless.create_worker(task).await).ok();
                            }
                        }
                    },
                    None => break, // sender dropped, shut down
                }
            },
        }
    }

    tracing::info!("shutting down pods...");

    // signal pods to stop here, then join
    for handle in handles {
        handle.abort();
        handle.await.ok();
    }
}
