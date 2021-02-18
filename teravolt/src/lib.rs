pub mod error;
pub mod executor;
pub mod packet;
pub mod types;

pub mod prelude {
    pub use crate::error::TeravoltError;
    pub use crate::executor::{Connection, Teravolt};
    pub use crate::packet::{Packet, TeravoltPacket};
    pub use crate::send_receive;
    pub use crate::types::*;
}

/// A `Result` type with a `TeravoltError` as the `Err` type.
pub type Result<T> = std::result::Result<T, error::TeravoltError>;

/// A convenience function to create a tokio unbounded channel for sending
/// packets.
pub fn send_receive() -> types::SendReceive {
    tokio::sync::mpsc::unbounded_channel()
}
