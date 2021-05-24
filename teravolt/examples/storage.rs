use teravolt::prelude::*;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};

#[derive(Debug, Clone)]
struct SendType;

#[teravolt::async_trait]
impl Connection<(), ()> for SendType {
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: Config<()>, _: MessageQueue, storage: Storage) -> TaskResult<()> {
        let mut interval = time::interval(Duration::from_millis(500));
        let data = storage.handle::<u16>().await;
        loop {
            interval.tick().await;
            *data.write().await += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct ReceiveType;

#[teravolt::async_trait]
impl Connection<(), ()> for ReceiveType {
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, _: Config<()>, _: MessageQueue, storage: Storage) -> TaskResult<()> {
        let mut interval = time::interval(Duration::from_millis(1000));
        let data = storage.handle::<u16>().await;
        loop {
            interval.tick().await;
            println!("Count: {}", data.read().await);
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

    let mut teravolt = Executor::new(&runtime, (), 0).unwrap();
    teravolt.add_connection(SendType).await;
    teravolt.add_connection(ReceiveType).await;
    teravolt.start().await;
}
