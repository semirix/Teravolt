use crate::{error::ErrorMessage, prelude::TeravoltError, Result};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast::*;

#[derive(Debug, Clone)]
/// A message queue structure
pub struct MessageQueue {
    handles: Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>>>,
    capacity: usize,
}

impl MessageQueue {
    /// Create a new message queue.
    #[tracing::instrument]
    pub fn new(capacity: usize) -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    /// Acquire a handle to one of the channels in the message queue system.
    #[tracing::instrument]
    pub fn handle<T: Any + Send + Sync + Clone + 'static>(
        &self,
    ) -> Result<(
        tokio::sync::broadcast::Sender<T>,
        tokio::sync::broadcast::Receiver<T>,
    )> {
        match self.handles.lock() {
            Ok(mut map) => {
                if let Some(handle) = map.get(&TypeId::of::<T>()) {
                    // Unwrapping this is fine because it theoretically should never
                    // panic.
                    let sender = handle
                        .downcast_ref::<tokio::sync::broadcast::Sender<T>>()
                        .unwrap();
                    let handle_sender = sender.clone();
                    let handle_receiver = sender.subscribe();

                    Ok((handle_sender, handle_receiver))
                } else {
                    let (sender, _) = channel::<T>(self.capacity);
                    let handle_sender = sender.clone();
                    let handle_receiver = sender.subscribe();
                    map.insert(TypeId::of::<T>(), Box::new(sender));

                    Ok((handle_sender, handle_receiver))
                }
            }
            Err(error) => Err(TeravoltError::GenericError(ErrorMessage(format!(
                "Could not acquire message handle - `{}`",
                error
            )))),
        }
    }
}
