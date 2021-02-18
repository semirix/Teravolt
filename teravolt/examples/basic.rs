use teravolt::prelude::*;
use tokio::runtime::{Builder, Handle};
use tokio::time::{self, Duration};

#[derive(Clone, TeravoltPacket)]
struct Type;

struct SendType;

impl Connection for SendType {
    fn name(&self) -> &'static str {
        "SendType"
    }
    fn task(&mut self, handle: &Handle, bus: Sender) -> TaskHandle {
        let handle = handle.spawn(async move {
            let mut interval = time::interval(Duration::from_millis(1000));
            loop {
                interval.tick().await;
                if let Err(_) = bus.send(Type.as_packet()) {
                    break;
                }
            }
        });

        (handle, None)
    }
}

struct ReceiveType;

impl Connection for ReceiveType {
    fn name(&self) -> &'static str {
        "ReceiveType"
    }
    fn task(&mut self, handle: &Handle, _: Sender) -> TaskHandle {
        let (sender, mut receiver) = send_receive();
        let handle = handle.spawn(async move {
            while let Some(message) = receiver.recv().await {
                if message.id() == Type::id() {
                    println!("Received a Type object");
                }
            }
        });
        (handle, Some(sender))
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
    let mut teravolt = Teravolt::new(&runtime).unwrap();
    teravolt.add_connection(SendType);
    teravolt.add_connection(ReceiveType);
    teravolt.start().await;
}
