/// This enum describes the behaviour of a Teravolt connection.
pub enum ConnectionBehaviour {
    /// A Producer doesn't read in messages from the main bus, it only produces
    /// values from the main bus.
    Producer,
    /// A Consumer reads in messages from the main bus but doesn't send messages
    /// back into it.
    Consumer,
    /// A Transformer reads messages and can send messages back to the bus. This
    /// is generall in response to a message coming from the bus.
    Transformer,
}

/// The ConnectionConfig determines certain characteristics of how the
/// connection should behave.
pub struct ConnectionConfig {
    /// This is the name of the connection. This must be unique otherwise a
    /// system can overwrite another.
    pub name: &'static str,
    /// This determines the behaviour of the connection and whether it is a
    /// Producer, Consumer, or Transformer.
    pub behaviour: ConnectionBehaviour,
}
