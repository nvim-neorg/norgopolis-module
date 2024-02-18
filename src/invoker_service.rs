use std::pin::Pin;

use crate::module_communication::{invoker_server::Invoker, Invocation, MessagePack};
use futures::Stream;
use tonic::{Request, Response, Status};

#[crate::async_trait]
pub trait Service {
    type Stream: Stream<Item = Result<MessagePack, Status>> + Send;

    async fn call(
        &self,
        fn_name: String,
        args: Option<MessagePack>,
    ) -> Result<Self::Stream, Status>;
}

pub struct InvokerService<T> {
    service: T,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
}

impl<T> InvokerService<T>
where
    T: Service,
{
    pub fn new(service: T, tx: tokio::sync::mpsc::UnboundedSender<()>) -> InvokerService<T> {
        InvokerService { service, tx }
    }
}

#[tonic::async_trait]
impl<T> Invoker for InvokerService<T>
where
    T: Service + Sync + Send + 'static,
{
    type InvokeStream = Pin<Box<dyn Stream<Item = Result<MessagePack, Status>> + Send>>;

    async fn invoke(
        &self,
        request: Request<Invocation>,
    ) -> Result<Response<Self::InvokeStream>, Status> {
        let invocation = request.into_inner();

        let _ = self.tx.send(());

        let response = self
            .service
            .call(invocation.function_name, invocation.args)
            .await?;

        Ok(Response::new(Box::pin(response)))
    }
}
