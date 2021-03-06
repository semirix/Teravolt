use crate::message::MessageQueue;
use crate::storage::Storage;
use crate::types::*;
use crate::Result;
use async_trait::async_trait;
use dyn_clone::DynClone;
use std::fmt::{self, Debug};
use std::sync::Arc;
use std::{any::TypeId, borrow::Borrow};
use tokio::task::JoinHandle;
use tokio::{runtime::Runtime, sync::RwLock};

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
pub trait Connection<E, C>: DynClone
where
    E: ErrorTrait,
    C: Send + Sync + Clone + Debug + 'static,
{
    /// A restart policy based upon the result of the task.
    fn policy(&self, result: TaskResult<E>) -> RestartPolicy;
    /// An asynchronous task for the  connection to run.
    async fn task(&self, config: Config<C>, queue: MessageQueue, storage: Storage)
        -> TaskResult<E>;
}

pub trait ErrorTrait: Send + Sync + 'static {}
impl<T> ErrorTrait for T where T: Send + Sync + 'static {}

/// The ConnectionContainer is a light wrapper around the Connection trait
/// object. Because it's essentially just a vtable, we don't need to worry
/// about mutability.
struct ConnectionContainer<E, C>
where
    E: ErrorTrait,
{
    raw: Box<dyn Connection<E, C> + Send + Sync>,
}

impl<E, C> fmt::Debug for ConnectionContainer<E, C>
where
    E: ErrorTrait,
    C: Send + Sync + Clone + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionContainer").finish()
    }
}

impl<E, C> ConnectionContainer<E, C>
where
    E: ErrorTrait,
    C: Send + Sync + Clone + Debug + 'static,
{
    #[tracing::instrument]
    fn new<T: Connection<E, C> + Send + Sync + Debug + 'static>(connection: T) -> Self {
        Self {
            raw: Box::new(connection),
        }
    }

    #[tracing::instrument]
    fn duplicate(&self) -> Self {
        Self {
            raw: dyn_clone::clone_box(&*self.raw),
        }
    }

    #[tracing::instrument]
    fn get_raw(&self) -> &Box<dyn Connection<E, C> + Send + Sync> {
        self.raw.borrow()
    }
}

/// A container for a global config object.
#[derive(Debug, Clone)]
pub struct Config<C>
where
    C: Send + Sync + Clone + Debug + 'static,
{
    config: Arc<RwLock<C>>,
}

impl<C> Config<C>
where
    C: Send + Sync + Clone + Debug + 'static,
{
    /// Create a new config object.
    #[tracing::instrument]
    pub fn new(config: C) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }
    /// Read the current config object.
    #[tracing::instrument]
    pub async fn get(&self) -> C {
        self.config.read().await.clone()
    }
    /// Set a new config object.
    #[tracing::instrument]
    pub async fn set(&mut self, config: C) -> Result<()> {
        *self.config.write().await = config;

        Ok(())
    }
}

#[tracing::instrument]
async fn connection_handle<E, C>(
    connection: ConnectionContainer<E, C>,
    config: Config<C>,
    queue: MessageQueue,
    storage: Storage,
) where
    E: ErrorTrait + Debug,
    C: Send + Sync + Clone + Debug + 'static,
{
    let connection = connection.get_raw();
    loop {
        let result = connection
            .task(config.clone(), queue.clone(), storage.clone())
            .await;

        match connection.policy(result) {
            RestartPolicy::Shutdown => break,
            RestartPolicy::Restart => (),
        }
    }
}

/// The Teravolt executor.
#[derive(Debug)]
pub struct Executor<'a, E, C>
where
    E: ErrorTrait + Debug,
    C: Send + Sync + Clone + Debug + 'static,
{
    runtime: &'a Runtime,
    connections: AsyncMap<TypeId, ConnectionContainer<E, C>>,
    tasks: AsyncMap<TypeId, JoinHandle<()>>,
    queue: MessageQueue,
    storage: Storage,
    config: Config<C>,
}

impl<'a, E, C> Executor<'a, E, C>
where
    E: ErrorTrait + Debug,
    C: Send + Sync + Clone + Debug + 'static,
{
    /// Create a new instance of the Teravolt executor.
    #[tracing::instrument]
    pub fn new(runtime: &'a Runtime, config: C, capacity: usize) -> Result<Self> {
        let queue = MessageQueue::new(capacity);
        Ok(Self {
            runtime,
            connections: async_map(),
            tasks: async_map(),
            queue: queue.clone(),
            storage: Storage::new(),
            config: Config::new(config),
        })
    }

    /// Add a new connection to the the Teravolt executor.
    pub async fn add_connection<T: Connection<E, C> + Send + Sync + Debug + 'static>(
        &mut self,
        connection: T,
    ) {
        self.connections
            .write()
            .await
            .insert(TypeId::of::<T>(), ConnectionContainer::new(connection));
    }

    /// Start the Teravolt executor.
    #[tracing::instrument]
    pub async fn start(&mut self) {
        let handle = self.runtime.handle();

        for (key, data) in self.connections.read().await.iter() {
            // Cloning the connection is fine, it's just a vtable. We need to
            // clone it so the thread can take ownership.
            let connection = data.duplicate();
            let handle_queue = self.queue.clone();
            let handle_storage = self.storage.clone();
            let config = self.config.clone();

            let thread = handle.spawn(connection_handle(
                connection,
                config,
                handle_queue,
                handle_storage,
            ));

            self.tasks.write().await.insert(*key, thread);
        }

        for (_, task) in self.tasks.write().await.iter_mut() {
            if let Err(error) = task.await {
                tracing::error!("Teravolt :: Error Joining Task: {}", error);
            }
        }
    }
}
