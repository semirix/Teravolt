use crate::message::Message;
use tokio::task::JoinHandle;

pub type Receiver = tokio::sync::mpsc::UnboundedReceiver<Message>;
pub type Sender = tokio::sync::mpsc::UnboundedSender<Message>;
pub type SendReceive = (Sender, Receiver);
pub type TaskHandle<E> = JoinHandle<TaskResult<E>>;
pub type TaskResult<E> = Result<(), E>;
