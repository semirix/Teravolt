use crate::message::Message;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub type Receiver = tokio::sync::mpsc::UnboundedReceiver<Message>;
pub type Sender = tokio::sync::mpsc::UnboundedSender<Message>;
pub type SendReceive = (Sender, Receiver);
pub type TaskHandle<E> = JoinHandle<TaskResult<E>>;
pub type TaskResult<E> = Result<(), E>;
pub type AsyncMap<K, V> = RwLock<HashMap<K, V>>;

/// A convenience function for creating an async map
pub fn async_map<K, V>() -> AsyncMap<K, V> {
    RwLock::new(HashMap::new())
}
