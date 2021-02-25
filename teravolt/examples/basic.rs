use async_trait::async_trait;
use teravolt::prelude::*;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};

#[derive(Clone, TeravoltMessage)]
struct Type;

#[derive(Clone)]
struct SendType;

#[async_trait]
impl Connection<()> for SendType {
    fn config(&self) -> ConnectionConfig {
        ConnectionConfig {
            name: "SendType",
            behaviour: ConnectionBehaviour::Producer,
        }
    }
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, sender: &Sender, _: &mut Receiver) -> TaskResult<()> {
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            interval.tick().await;
            if let Err(_) = sender.send(Type.as_message()) {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct ReceiveType;

#[teravolt::async_trait]
impl Connection<()> for ReceiveType {
    fn config(&self) -> ConnectionConfig {
        ConnectionConfig {
            name: "ReceiveType",
            behaviour: ConnectionBehaviour::Consumer,
        }
    }
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: &Sender, receiver: &mut Receiver) -> TaskResult<()> {
        while let Some(message) = receiver.recv().await {
            if message.id() == Type::id() {
                println!("Received a Type object");
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let runtime = Builder::new_multi_thread()
        .thread_name("teravolt-worker")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_time()
        .build()
        .unwrap();

    let mut teravolt = Executor::new(&runtime).unwrap();
    teravolt.add_connection(SendType).await;
    teravolt.add_connection(ReceiveType).await;
    teravolt.start().await;
}
