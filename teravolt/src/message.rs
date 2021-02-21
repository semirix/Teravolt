use std::any::{Any, TypeId};
use std::sync::Arc;

pub use teravolt_codegen::TeravoltMessage;

/// A trait for a Teravolt Message. This trait should be implemented using the
/// `#[derive(Teravolt)]` macro.
pub trait TeravoltMessage {
    fn as_message(&self) -> Message;
    fn id() -> TypeId;
}

#[derive(Debug, Clone)]
/// A universal message type for transmitting data inside Teravolt.
pub struct Message(Arc<Box<dyn Any + Send + Sync>>, TypeId);

impl Message {
    /// Creates a new Message. Avoid calling this method directly, this is for
    /// use by the `#[derive(TeravoltMessage)]` derive macro.
    pub fn new<T: Any + Clone + Send + Sync>(data: T) -> Self {
        let data = data.clone();
        Message(Arc::new(Box::new(data)), TypeId::of::<T>())
    }
    /// Clone the data within the message and consume the lifetime of the
    /// message. Don't use this unless you know what the type beforehand is.
    pub fn consume<T: Any + Clone + Send + Sync>(self) -> Option<T> {
        match self.0.downcast_ref::<T>() {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    /// Get immutable reference to data.
    pub fn get<T: Any + Clone + Send + Sync>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }

    /// A convenience function for getting the type id of a message,
    pub fn id(&self) -> TypeId {
        self.1
    }
}
