use hashbrown::HashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::broadcast::*;
use tokio::sync::Mutex;

#[derive(Clone)]
/// A message queue structure
pub struct MessageQueue {
    handles: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>>>,
    capacity: usize,
}

impl MessageQueue {
    /// Create a new message queue.
    pub fn new(capacity: usize) -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    /// Acquire a handle to one of the channels in the message queue system.
    pub async fn handle<T: Any + Send + Sync + Clone + 'static>(
        &self,
    ) -> (
        tokio::sync::broadcast::Sender<T>,
        tokio::sync::broadcast::Receiver<T>,
    ) {
        let mut map = self.handles.lock().await;
        if let Some(handle) = map.get(&TypeId::of::<T>()) {
            // Unwrapping this is fine because it theoretically should never
            // panic.
            let (sender, _) = handle
                .downcast_ref::<(
                    tokio::sync::broadcast::Sender<T>,
                    tokio::sync::broadcast::Receiver<T>,
                )>()
                .unwrap();
            let handle_sender = sender.clone();
            let handle_receiver = sender.subscribe();

            (handle_sender, handle_receiver)
        } else {
            let (sender, receiver) = channel::<T>(self.capacity);
            let handle_sender = sender.clone();
            let handle_receiver = sender.subscribe();
            map.insert(TypeId::of::<T>(), Box::new((sender, receiver)));

            (handle_sender, handle_receiver)
        }
    }
}
