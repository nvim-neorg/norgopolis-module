pub mod invoker_service;
mod stdio_service;

use std::time::Duration;

use futures::FutureExt;
use invoker_service::InvokerService;
use invoker_service::Service;
use module_communication::invoker_server::InvokerServer;
use stdio_service::StdioService;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;

pub use norgopolis_protos::module_communication;
pub use tonic::async_trait;
pub use tonic::{Code, Status};

pub trait Shutdown {
    fn shutdown(&self);
}

pub struct Module {
    pub timeout: Duration,
}

impl Module {
    pub fn new() -> Self {
        Module {
            timeout: Duration::from_secs(60 * 5),
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        Module { timeout }
    }

    pub async fn start<S>(self, service: S) -> Result<(), anyhow::Error>
    where
        S: Service + Shutdown + Sync + Send + 'static,
    {
        let (keepalive_tx, mut keepalive_rx) = tokio::sync::mpsc::channel::<()>(1);

        tokio::spawn(async move {
            sleep(self.timeout).await;

            if keepalive_rx.recv().now_or_never().is_none() {
                std::process::exit(0);
            }
        });

        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        let stdio_service = StdioService { stdin, stdout };

        // TODO: Do this in a better way
        // `once_stream` doesn't work :/
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StdioService, anyhow::Error>>(1);
        tx.send(Ok(stdio_service)).await?;

        Ok(Server::builder()
            .add_service(InvokerServer::new(InvokerService::new(
                service,
                keepalive_tx,
            )))
            .serve_with_incoming(ReceiverStream::new(rx))
            .await?)
    }
}
