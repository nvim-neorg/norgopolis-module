mod stdio_service;
pub mod invoker_service;

use invoker_service::InvokerService;
use invoker_service::Service;
use module_communication::invoker_server::InvokerServer;
use stdio_service::StdioService;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;

pub use tonic::async_trait;
pub use tonic::{Code, Status};
pub use norgopolis_protos::module_communication;

pub struct Module {}

impl Module {
    pub async fn start<S>(service: S) -> Result<(), anyhow::Error>
    where
        S: Service + Sync + Send + 'static,
    {
        // TODO: Make configurable
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        let stdio_service = StdioService { stdin, stdout };

        // TODO: Do this in a better way
        // `once_stream` doesn't work :/
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StdioService, anyhow::Error>>(1);
        tx.send(Ok(stdio_service)).await?;

        Ok(Server::builder()
            .add_service(InvokerServer::new(InvokerService::new(service)))
            .serve_with_incoming(ReceiverStream::new(rx))
            .await?)
    }
}
