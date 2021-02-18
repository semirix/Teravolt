use crate::types::*;
use crate::Result;
use dashmap::DashMap;
use tokio::runtime::{Handle, Runtime};

pub trait Connection {
    fn name(&self) -> &'static str;
    fn task(&mut self, handle: &Handle, bus: Sender) -> TaskHandle;
}

/// The Teravolt executor.
pub struct Teravolt<'a> {
    runtime: &'a Runtime,
    primary_send: Sender,
    primary_receive: Receiver,
    connections: DashMap<&'static str, Box<dyn Connection + Send + Sync>>,
    tasks: DashMap<&'static str, TaskHandle>,
}
impl<'a> Teravolt<'a> {
    /// Create a new instance of the Teravolt executor.
    pub fn new(runtime: &'a Runtime) -> Result<Self> {
        let (send, receive) = tokio::sync::mpsc::unbounded_channel();
        Ok(Self {
            runtime,
            primary_send: send,
            primary_receive: receive,
            connections: DashMap::new(),
            tasks: DashMap::new(),
        })
    }

    /// Add a new connection to the the Teravolt executor.
    pub fn add_connection<T: Connection + Send + Sync + 'static>(&mut self, connection: T) {
        self.connections
            .insert(connection.name(), Box::new(connection));
    }

    /// Start the
    pub async fn start(&mut self) {
        let handle = self.runtime.handle();

        for mut connection in self.connections.iter_mut() {
            // Spawn the task
            let join_task = connection.task(handle, self.primary_send.clone());
            self.tasks.insert(connection.name(), join_task);
        }

        while let Some(message) = self.primary_receive.recv().await {
            for task in self.tasks.iter() {
                if let Some(sender) = &task.1 {
                    match sender.send(message.clone()) {
                        Err(error) => println!("Error: {error}", error = error),
                        _ => (),
                    }
                }
            }
        }
    }
}
