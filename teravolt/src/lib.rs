//! Teravolt is an executor for handling streaming data from multiple sources
//! and enabling seamless communication between them.
//!
//! ## Getting started
//! There are two main components to getting Teravolt up and running:
//! - [`Connection`][executor::Connection]
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
//! It is very important that our struct is cloneable. Next
//! we need to implement the actual [`Connection`]. When implementing a
//! connection we need to create a global error type that will be used across
//! all connections running in the executor. An implementation for a
//! [`Connection`] will look like this:
//!
//! ```rust
//! # use teravolt::prelude::*;
//! # use tokio::time::{self, Duration};
//! # #[derive(Clone)]
//! # struct SendType;
//! # #[derive(Clone)]
//! # struct Type;
//!
//! #[teravolt::async_trait]
//! impl Connection<(), ()> for SendType {
//!     fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
//!         RestartPolicy::Restart
//!     }
//!     async fn task(&self, _: Config<()>, queue: MessageQueue, _: Storage) -> TaskResult<()> {
//!         let (sender, _) = queue.handle::<Type>().await;
//!         let mut interval = time::interval(Duration::from_millis(1000));
//!         loop {
//!             interval.tick().await;
//!             if let Err(_) = sender.send(Type) {
//!                 break;
//!             }
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### Restart Policy
//! The restart policy will determine what happens when the task either
//! completes or fails. It takes the result produced by the task and it is up to
//! you to determine what circumstances will cause a Restart or a Shutdown.
//!
//! ### Task
//! Each connection has an asynchronous task that is responsible for reading in
//! messages and sending them to other connections.
//!
//! A task has the ability to read messages sent from other connections and send
//! messages from other connections.
//!
//! ## Executor
//! The executor is the main manager for all of the connections you'll be
//! running. Setting up the executor is quite simple, first you need to create
//! a tokio runtime with the right features enabled for your use-case:
//!
//! ```rust
//! # use tokio::runtime::{Builder, Runtime};
//! # fn main() {
//! let runtime = Builder::new_multi_thread()
//!     .thread_name("teravolt-worker")
//!     .thread_stack_size(4 * 1024 * 1024)
//!     .enable_time()
//!     .build()
//!     .unwrap();
//! # }
//! ```
//!
//! Then once you've made the runtime you can create an instance of the
//! executor, add your connections, your config, set the maximum message
//! capacity, and you're good to go!
//!
//! ```rust
//! # use teravolt::prelude::*;
//! # use tokio::time::{self, Duration};
//! # #[derive(Clone)]
//! # struct SendType;
//! # #[derive(Clone)]
//! # struct Type;
//!
//! # #[teravolt::async_trait]
//! # impl Connection<(), ()> for SendType {
//! #     fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
//! #         RestartPolicy::Restart
//! #     }
//! #     async fn task(&self, _: Config<()>, queue: MessageQueue, _: Storage) -> TaskResult<()> {
//! #         let (sender, _) = queue.handle::<Type>().await;
//! #         let mut interval = time::interval(Duration::from_millis(1000));
//! #         loop {
//! #             interval.tick().await;
//! #             if let Err(_) = sender.send(Type) {
//! #                 break;
//! #             }
//! #         }
//! #         Ok(())
//! #     }
//! # }
//! # #[tokio::main]
//! # async fn main() {
//! # use tokio::runtime::{Builder, Runtime};
//! # let runtime = Builder::new_multi_thread()
//! #    .thread_name("teravolt-worker")
//! #    .thread_stack_size(4 * 1024 * 1024)
//! #    .enable_time()
//! #    .build()
//! #    .unwrap();
//! let mut teravolt = Executor::new(&runtime, (), 32).unwrap();
//!     teravolt.add_connection(SendType);
//!     teravolt.start().await;
//! # runtime.shutdown_background();
//! # }

//! ```
pub mod error;
pub mod executor;
pub mod message;
pub mod storage;
pub mod types;

pub mod prelude {
    pub use crate::error::TeravoltError;
    pub use crate::executor::{Config, Connection, Executor, RestartPolicy};
    pub use crate::message::MessageQueue;
    pub use crate::storage::Storage;
    pub use crate::types::*;
}

pub use crate::executor::Connection;
pub use crate::executor::Executor;

/// A `Result` type with a `TeravoltError` as the `Err` type.
pub type Result<T> = std::result::Result<T, error::TeravoltError>;

/// A re-export of [`async-trait`] for convenience.
pub use async_trait::async_trait;
