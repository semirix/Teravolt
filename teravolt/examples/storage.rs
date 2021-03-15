use teravolt::prelude::*;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};

#[derive(Clone)]
struct SendType;

#[teravolt::async_trait]
impl Connection<()> for SendType {
    fn config(&self) -> ConnectionConfig {
        ConnectionConfig { name: "SendType" }
    }
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: MessageQueue, storage: Storage) -> TaskResult<()> {
        let mut interval = time::interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            storage
                .write::<u16>(|data| {
                    *data += 1;
                })
                .await;
        }
    }
}

#[derive(Clone)]
struct ReceiveType;

#[teravolt::async_trait]
impl Connection<()> for ReceiveType {
    fn config(&self) -> ConnectionConfig {
        ConnectionConfig {
            name: "ReceiveType",
        }
    }
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: MessageQueue, storage: Storage) -> TaskResult<()> {
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            interval.tick().await;
            storage
                .read::<u16>(|data| {
                    println!("Count: {}", data);
                })
                .await;
        }
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

    let mut teravolt = Executor::new(&runtime, 0).unwrap();
    teravolt.add_connection(SendType).await;
    teravolt.add_connection(ReceiveType).await;
    teravolt.start().await;
}
