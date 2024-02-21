//! # A Library for Creating Norgopolis Modules
//! 
//! For information about Norgopolis, consult https://github.com/nvim-neorg/norgopolis.
//! 
//! This library exposes an API for creating and maintaining a connection to the Norgopolis router.
//! Norgopolis modules provide specific sets of functionality, for example multithreaded parsing, database
//! access, etc. All of the default modules created by the Neorg team are built on top of this library.
//!
//! # Usage
//! 
//! ### General Setup
//! 
//! First, create a struct for your module. Name it whatever you'd like:
//! 
//! ```rs
//! use norgopolis_module::{
//!     invoker_service::Service, module_communication::MessagePack, Code, Module, Status,
//! };
//! 
//! #[derive(Default)]
//! struct MyModule {
//!     // add any data or state you might need to maintain here...
//! }
//! ```
//! 
//! Second, implement the `norgopolis_module::invoker_service::Service` trait for your struct.
//! This forces you to implement a `call` function which will be invoked any time someone routes
//! a message to your module. Since async traits are not stabilized within Rust yet, tag your
//! trait implementation with `#[norgopolis_module::async_trait]`:
//! 
//! ```rs
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! 
//! #[norgopolis_module::async_trait]
//! impl Service for MyModule {
//!     type Stream = UnboundedReceiverStream<Result<MessagePack, Status>>;
//! 
//!     async fn call(
//!         &self,
//!         function: String,
//!         args: Option<MessagePack>,
//!     ) -> Result<Self::Stream, Status> {
//!         todo!()
//!     }
//! }
//! ```
//! 
//! ##### `Stream`
//! 
//! The `Stream` type defines what sort of data will be returned back via gRPC. We recommend
//! that you set it to `UnboundedReceiverStream<Result<MessagePack, Status>>`. This means that
//! given one request your module will be able to return an infinite amount of MessagePack responses,
//! or a status code in case something went wrong.
//! 
//! ##### `call`
//! 
//! The `call` function gets invoked whenever a client routes a message to you. The message contains:
//! - The function that they would like to invoke
//! - An optional set of parameters they would like to supply to the function.
//! 
//! ### Creating the Basic Glue
//! 
//! In the `call` function it's recommended to match over all possible function names that your module
//! supports and returning an error code if it's unsupported:
//! 
//! ```rs
//! match function.as_str() {
//!     "my-function" => todo!(),
//!     _ => Err(Status::new(Code::NotFound, "Requested function not found!")),
//! }
//! ```
//! 
//! > [!IMPORTANT]
//! > It's always better to return *some* sort of status code over panicking.
//! > Panicking will terminate the connection to Norgopolis and the user will not receive
//! > any sort of error or warning.
//! 
//! ### Decoding the Parameters
//! 
//! If your function takes in any amount of parameters then now is the time to decode them.
//! If your parameter is complex (e.g. a dictionary) then it's recommended to create a struct
//! designated for it. Be sure to derive `serde::Serialize`:
//! 
//! ```rs
//! #[derive(serde::Serialize)]
//! struct MyParameters {
//!     name: String,
//! }
//! ```
//! 
//! Aftewards, it's a simple matter of running `decode` on your arguments:
//! 
//! ```rs
//! match function.as_str() {
//!     "my-function" => {
//!         let args: MyParameters = args
//!             .unwrap() // WARNING: Don't actually use unwrap() in your code :)
//!             .decode()
//!             .map_err(|err| Status::new(Code::InvalidArgument, err.to_string()))?;
//! 
//!         // TODO: Do something with the parameters...
//!     },
//! }
//! ```
//! 
//! We manually provide the type of `args` so that Rust knows what type to serialize to.
//! Afterwards we wrap any possible errors into a status code which can be returned back to the client.
//! 
//! ### Sending Data back to the Client
//! 
//! Now that we have all of the input data in check we can process our data and return it back to the client.
//! The way we do this is in the form of a data stream. Thanks to data streams we can return long segments of
//! data over time instead of having to return the whole data upfront. When we return a segment of data, we
//! also return it in the form of a `Result<>`. This is because individual segments of data may contain errors,
//! but the whole process can complete succesfully. You should return errors from the `call` function when there
//! is an irrecoverable error, but should send back an error packet when a *portion* of the internal logic fails.
//! 
//! Let's showcase all of this via an example:
//! 
//! ```rs
//! match function.as_str() {
//!     "my-function" => {
//!         let args: MyParameters = args
//!             .unwrap() // WARNING: Don't actually use unwrap() in your code :)
//!             .decode()
//!             .map_err(|err| Status::new(Code::InvalidArgument, err.to_string()))?;
//! 
//!         let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//! 
//!         // We send back an Ok() packet to the client with an encoded message of our choice
//!         // (it can be anything that's serializable with serde!)
//!         tx.send(Ok(MessagePack::encode(format!("Hello, {}!", args.name)))).unwrap();
//! 
//!         Ok(UnboundedReceiverStream::new(rx))
//!     },
//! }
//! ```
//! 
//! First, we create a sender and receiver via tokio's `unbounded_channel()`. This allows us to send data to the client
//! and for the client to read data from the module. All return messages have to be encoded via `MessagePack::encode`.
//! 
//! ### Running the Module
//! 
//! Now that we have all of the code set up, create an asynchronous main function. In here we will instantiate our
//! module and kick it into full gear:
//! 
//! ```rs
//! #[tokio::main]
//! async fn main() {
//!     Module::new().start(MyModule::default())
//!         .await
//!         .unwrap()
//! }
//! ```
//! 
//! Voila! You now have a fundamental understanding of how modules communicate with Norgopolis and how to write your own
//! norgopolis module. Happy coding!

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

/// Describes a module that can communicate with Norgopolis
/// over stdin/stdout.
pub struct Module {
    /// Timeout duration for the module. If no messages are received by the module after this time
    /// has passed the module will automatically shut down.
    ///
    /// Default is 5 minutes.
    pub timeout: Duration,
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
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
        S: Service + Sync + Send + 'static,
    {
        let (keepalive_tx, mut keepalive_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        tokio::spawn(async move {
            sleep(self.timeout).await;

            if keepalive_rx.recv().now_or_never().is_none() {
                std::process::exit(0);
            }

            // Drain the remained of the messages.
            while keepalive_rx.recv().now_or_never().is_some() {}
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
