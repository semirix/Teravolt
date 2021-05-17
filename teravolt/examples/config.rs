use teravolt::prelude::*;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};

#[derive(Debug, Clone)]
struct Type;

#[derive(Clone)]
struct SendType;

#[teravolt::async_trait]
impl Connection<(), String> for SendType {
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(
        &self,
        mut config: Config<String>,
        _: MessageQueue,
        _: Storage,
    ) -> TaskResult<()> {
        let mut counter = 0;
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            interval.tick().await;
            counter += 1;
            config.set(format!("I am a: {}", counter)).await.unwrap();
        }
    }
}

#[derive(Clone)]
struct ReceiveType;

#[teravolt::async_trait]
impl Connection<(), String> for ReceiveType {
    fn policy(&self, _: TaskResult<()>) -> RestartPolicy {
        RestartPolicy::Restart
    }
    async fn task(&self, config: Config<String>, _: MessageQueue, _: Storage) -> TaskResult<()> {
        let mut interval = time::interval(Duration::from_millis(1000));
        loop {
            interval.tick().await;
            println!("{}", config.get().await);
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

    let mut teravolt = Executor::new(&runtime, "I am first.".into(), 32).unwrap();
    teravolt.add_connection(SendType).await;
    teravolt.add_connection(ReceiveType).await;
    teravolt.start().await;
}
