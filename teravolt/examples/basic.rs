use teravolt::prelude::*;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};

#[derive(Debug, Clone)]
struct Type;

#[derive(Debug, Clone)]
struct SendType;

#[teravolt::async_trait]
impl Connection<(), ()> for SendType {
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: Config<()>, queue: MessageQueue, _: Storage) -> TaskResult<()> {
        let (sender, _) = queue.handle::<Type>().unwrap();
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            interval.tick().await;
            if let Err(_) = sender.send(Type) {
                break;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ReceiveType;

#[teravolt::async_trait]
impl Connection<(), ()> for ReceiveType {
    fn policy(&self, __: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: Config<()>, queue: MessageQueue, _: Storage) -> TaskResult<()> {
        let (_, mut receiver) = queue.handle::<Type>().unwrap();
        while let Ok(message) = receiver.recv().await {
            println!("Received a {:?} object", message);
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

    let mut teravolt = Executor::new(&runtime, (), 32).unwrap();
    teravolt.add_connection(SendType).await;
    teravolt.add_connection(ReceiveType).await;
    teravolt.start().await;
}
