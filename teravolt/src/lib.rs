//! Teravolt is an executor for handling streaming data from multiple sources
//! and enabling seamless communication between them.
//!
//! ## Getting started
//! There are three main components to getting Teravolt up and running:
//! - [`Connection`][executor::Connection]
//! - [`Message`][message]
//! - [`Executor`][executor]
//!
//! ## Creating a Connection
//! To create a new connection, we need to create a blank struct, like so:
//!
//! ```rust
//! #[derive(Clone)]
//! struct SendMessage;
//! ```
//!
//! It is very important that our struct is cloneable. Next we need to implement
//! the actual [`Connection`]. When implementing a connection we need to create
//! a global error type that will be used across all connections running in the
//! executor. This can reasonably be anything but for now, `()` will be adequate
//! for demonstration. An implementation for a [`Connection`] will look like
//! this:
//!
//! ```rust
//! #[teravolt::async_trait]
//! impl Connection<Error> for SendMessage {
//!     fn config(&self) -> ConnectionConfig {
//!         // ...
//!     }
//!     fn policy(&self, _: TaskResult<Error>) -> RestartPolicy {
//!         // ...
//!     }
//!     async fn task(&self, sender: &Sender, _: &mut Receiver) -> TaskResult<Error> {
//!         // ...
//!     }
//! }
//! ```
//!
//! ### Config
//! This is how we determine the behaviour of the connection. For instance, a
//! config will look this:
//!
//! ```rust
//! fn config(&self) -> ConnectionConfig {
//!     ConnectionConfig {
//!         name: "SendMessage",
//!         behaviour: ConnectionBehaviour::Producer,
//!     }
//! }
//! ```
//!
//! Here we define the name of the connection and it's behaviour. The name of
//! the connection must be unique inside the executor. The behaviour determines
//! how the connection will deal with receiving and sending messages. There is a
//! Producer, Consumer, and Transformer behaviour type. To learn more about what
//! these do, read the [`ConnectionBehaviour`][config::ConnectionBehaviour]
//! documentation.
//!
//! ### Restart Policy
//! The restart policy will determine what happens when the task either
//! completes or fails. It takes the result produced by the task and it is up to
//! you to determine what circumstances will cause a Restart or a Shutdown. A
//! restart policy will sometimes look like this:
//!
//! ```rust
//! fn policy(&self, result: TaskResult<Error>) -> RestartPolicy {
//!     if let Err(error) = result {
//!         RestartPolicy::Restart
//!     } else {
//!         RestartPolicy::Shutdown
//!     }
//! }
//! ```
//!
//! ### Task
//! Each connection has an asynchronous task that is responsible for reading in
//! messages and sending them to other connections. A task will sometimes look
//! like this:
//!
//! ```rust
//! async fn task(&self, sender: &Sender, _: &mut Receiver) -> TaskResult<()> {
//!     let mut interval = time::interval(Duration::from_millis(1000));
//!     loop {
//!         interval.tick().await;
//!         if let Err(_) = sender.send(Type.as_message()) {
//!             break;
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! A task has the ability to read messages sent from other connections and send
//! messages from other connections. The ability to read and write is determined
//! by the connection's [`ConnectionBehaviour`][config::ConnectionBehaviour].
//!
//! ## Messages
//! In Teravolt, a [`Message`] is a universal type for sending data between
//! connections. To allow your types to be sent as a message, do the following:
//!
//! ```rust
//! #[derive(Clone, TeravoltMessage)]
//! struct MyType;
//! ```
//!
//! Make sure that your type implements [`Clone`], otherwise you will get an
//! error.
//!
//! ## Executor
//! The executor is the main manager for all of the connections you'll be
//! running. Setting up the executor is quite simple, first you need to create
//! a tokio runtime with the right features enabled for your use-case:
//!
//! ```rust
//! let runtime = Builder::new_multi_thread()
//!     .thread_name("teravolt-worker")
//!     .thread_stack_size(3 * 1024 * 1024)
//!     .enable_time()
//!     .build()
//!     .unwrap();
//! ```
//!
//! Then once you've made the runtime you can create an instance of the
//! executor, add your connections, and then start it!
//!
//! ```rust
//! let mut teravolt = Executor::new(&runtime).unwrap();
//!     teravolt.add_connection(MyConnection);
//!     teravolt.start().await;
//! ```
pub mod config;
pub mod error;
pub mod executor;
pub mod message;
pub mod types;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::error::TeravoltError;
    pub use crate::executor::{Connection, Executor, RestartPolicy};
    pub use crate::message::{Message, TeravoltMessage};
    pub use crate::types::*;
}

pub use crate::executor::Connection;
pub use crate::executor::Executor;
pub use crate::message::Message;

/// A `Result` type with a `TeravoltError` as the `Err` type.
pub type Result<T> = std::result::Result<T, error::TeravoltError>;

/// A re-export of [`async-trait`] for convenience.
pub use async_trait::async_trait;
