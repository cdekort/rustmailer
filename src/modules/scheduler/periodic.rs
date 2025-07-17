use crate::modules::{common::signal::SIGNAL_MANAGER, error::RustMailerResult};
use std::{future::Future, time::Duration};
use tokio::sync::oneshot;
use tracing::{info, warn};

pub struct PeriodicTask {
    name: String,
}

pub struct TaskHandle {
    cancel_sender: Option<oneshot::Sender<()>>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl TaskHandle {
    pub async fn cancel(self) {
        if let Some(sender) = self.cancel_sender {
            let _ = sender.send(());
        }
        let _ = self.join_handle.await;
    }
}

impl PeriodicTask {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }

    /// If `enable_cancel` is true, allows cancellation through TaskHandle::cancel
    pub fn start<F, T>(
        self,
        task: T,
        param: Option<u64>,
        interval: Duration,
        enable_cancel: bool,
        run_immediately: bool,
    ) -> TaskHandle
    where
        T: Fn(Option<u64>) -> F + Send + Sync + 'static,
        F: Future<Output = RustMailerResult<()>> + Send + 'static,
    {
        info!("Task '{}' started", &self.name);

        let (cancel_sender_opt, cancel_receiver_opt) = if enable_cancel {
            let (tx, rx) = oneshot::channel::<()>();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };

        let name_clone = self.name.clone();

        let join_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            let mut shutdown = SIGNAL_MANAGER.subscribe();

            if !run_immediately {
                interval.tick().await; // discard first immediate tick
            }
            let mut cancel_receiver = cancel_receiver_opt;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match task(param).await {
                            Ok(()) => {},
                            Err(e) => {
                                warn!("Task '{}' failed: {:?}", name_clone, e);
                            },
                        }
                    }
                    // only enabled if cancel_receiver is Some
                    _ = async {
                        if let Some(ref mut rx) = cancel_receiver {
                            rx.await.ok()
                        } else {
                            futures::future::pending().await
                        }
                    } => {
                        info!("Task '{}' received cancellation signal", name_clone);
                        break;
                    }
                    _ = shutdown.recv() => {
                        info!("Task '{}' shutting down due to shutdown signal", name_clone);
                        break;
                    }
                }
            }

            info!("Task '{}' stopped", name_clone);
        });

        TaskHandle {
            cancel_sender: cancel_sender_opt,
            join_handle,
        }
    }
}
