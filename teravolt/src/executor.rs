use crate::config::*;
use crate::types::*;
use crate::Result;
use async_trait::async_trait;
use dyn_clone::DynClone;
use std::borrow::Borrow;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

/// An enum returned by [`Connection::policy`] in response to a task error that
/// determines if the task needs to shutdown.
pub enum RestartPolicy {
    /// Restart the task
    Restart,
    /// Shutdown the task
    Shutdown,
}

/// The connection trait for Teravolt. You will implement this directly on a
/// blank cloneable object. When implementing, make sure you use the
/// `#[teravolt::async_trait]` macro.
#[async_trait]
pub trait Connection<E>: DynClone
where
    E: ErrorTrait,
{
    /// The static config for the connection.
    fn config(&self) -> ConnectionConfig;
    /// A restart policy based upon the result of the task.
    fn policy(&self, result: TaskResult<E>) -> RestartPolicy;
    /// An asynchronous task for the  connection to run.
    async fn task(&self, sender: &Sender, receiver: &mut Receiver) -> TaskResult<E>;
}

pub trait ErrorTrait: Send + Sync + 'static {}
impl<T> ErrorTrait for T where T: Send + Sync + 'static {}

/// The ConnectionContainer is a light wrapper around the Connection trait
/// object. Because it's essentially just a vtable, we don't need to worry
/// about mutability.
struct ConnectionContainer<E>
where
    E: ErrorTrait,
{
    raw: Box<dyn Connection<E> + Send + Sync>,
}

impl<E> ConnectionContainer<E>
where
    E: ErrorTrait,
{
    fn new<T: Connection<E> + Send + Sync + 'static>(connection: T) -> Self {
        Self {
            raw: Box::new(connection),
        }
    }

    fn duplicate(&self) -> Self {
        Self {
            raw: dyn_clone::clone_box(&*self.raw),
        }
    }

    fn get_raw(&self) -> &Box<dyn Connection<E> + Send + Sync> {
        self.raw.borrow()
    }
}

/// The Teravolt executor.
pub struct Executor<'a, E>
where
    E: ErrorTrait,
{
    runtime: &'a Runtime,
    primary_sender: Sender,
    primary_receiver: Receiver,
    connections: AsyncMap<&'static str, ConnectionContainer<E>>,
    tasks: AsyncMap<&'static str, JoinHandle<()>>,
    senders: AsyncMap<&'static str, Sender>,
}

impl<'a, E> Executor<'a, E>
where
    E: ErrorTrait,
{
    /// Create a new instance of the Teravolt executor.
    pub fn new(runtime: &'a Runtime) -> Result<Self> {
        let (primary_sender, primary_receiver) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            runtime,
            primary_sender,
            primary_receiver,
            connections: async_map(),
            tasks: async_map(),
            senders: async_map(),
        })
    }

    /// Add a new connection to the the Teravolt executor.
    pub async fn add_connection<T: Connection<E> + Send + Sync + 'static>(
        &mut self,
        connection: T,
    ) {
        let config = connection.config();
        self.connections
            .write()
            .await
            .insert(config.name, ConnectionContainer::new(connection));
    }

    /// Start the Teravolt executor.
    pub async fn start(&mut self) {
        let handle = self.runtime.handle();

        for (_, data) in self.connections.read().await.iter() {
            // Cloning the connection is fine, it's just a vtable. We need to
            // clone it so the thread can take ownership.
            let connection = data.duplicate();
            let config = connection.get_raw().config();
            // Spawn the task
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
            let bus = self.primary_sender.clone();

            let thread = handle.spawn(async move {
                let connection = connection.get_raw();
                loop {
                    let result = connection.task(&bus, &mut receiver).await;

                    match connection.policy(result) {
                        RestartPolicy::Shutdown => break,
                        RestartPolicy::Restart => (),
                    }
                }
            });

            self.tasks.write().await.insert(config.name, thread);

            match config.behaviour {
                ConnectionBehaviour::Consumer | ConnectionBehaviour::Transformer => {
                    self.senders.write().await.insert(config.name, sender);
                }
                _ => (),
            }
        }

        while let Some(message) = self.primary_receiver.recv().await {
            for (_, sender) in self.senders.read().await.iter() {
                match sender.send(message.clone()) {
                    // TODO: Handle this better
                    Err(error) => println!("Error: {error}", error = error),
                    _ => (),
                }
            }
        }
    }
}
