use std::any::{Any, TypeId};
use std::sync::Arc;

pub use teravolt_codegen::TeravoltPacket;

/// A trait for a Teravolt Packet. This trait should be implemented using the
/// `#[derive(Teravolt)]` macro.
pub trait TeravoltPacket {
    fn as_packet(&self) -> Packet;
    fn id() -> TypeId;
}

#[derive(Debug, Clone)]
/// A universal packet type for transmitting data inside Teravolt.
pub struct Packet(Arc<Box<dyn Any + Send + Sync>>, TypeId);

impl Packet {
    /// Creates a new Packet. Avoid calling this method directly, this is for
    /// use by the `#[derive(Packet)]` derive macro.
    pub fn new<T: Any + Clone + Send + Sync>(data: T) -> Self {
        let data = data.clone();
        Packet(Arc::new(Box::new(data)), TypeId::of::<T>())
    }
    /// Clone the data within the packet and consume the lifetime of the packet
    /// Don't use this unless you know what the type beforehand is.
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

    /// A convenience function for getting the type id of a packet,
    pub fn id(&self) -> TypeId {
        self.1
    }
}
