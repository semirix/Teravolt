/// This enum describes the behaviour of a Teravolt connection.
pub enum ConnectionBehaviour {
    /// A Producer doesn't read in messages it only produces new messages.
    Producer,
    /// A Consumer reads in messages but doesn't send messages.
    Consumer,
    /// A Transformer reads messages and can send messages. This is the most
    /// common behaviour.
    Transformer,
}

/// The ConnectionConfig determines certain characteristics of how the
/// connection should behave.
pub struct ConnectionConfig {
    /// This is the name of the connection. This must be unique.
    pub name: &'static str,
    /// This determines the behaviour of the connection and whether it is a
    /// Producer, Consumer, or Transformer.
    pub behaviour: ConnectionBehaviour,
}
