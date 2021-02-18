use crate::packet::Packet;
use tokio::task::JoinHandle;

pub type Receiver = tokio::sync::mpsc::UnboundedReceiver<Packet>;
pub type Sender = tokio::sync::mpsc::UnboundedSender<Packet>;
pub type SendReceive = (Sender, Receiver);
pub type TaskHandle = (JoinHandle<()>, Option<Sender>);
