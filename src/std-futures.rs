#![allow(unused)]
#![feature(async_await)]

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    sync::mpsc::{channel, Sender},
    task::{Context, Waker, Poll},
    thread,
    time::Duration,
    io,
};

use tokio;

use futures::executor;

pub struct ConsumerFuture {
    shared_state: Arc<Mutex<SharedState>>,
    sender: Option<Sender<u16>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for ConsumerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl ConsumerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        let (sender, receiver) = channel();

        thread::spawn(move || {
            let tid = thread::current().id();
            loop {
                match receiver.recv() {
                    Ok(option) => {
                        match option {
                            Some(elem) => println!("{:?} consume element: {:?}", tid, elem),
                            None => {
                                println!("{:?} consume done", tid);
                                let mut shared_state = thread_shared_state.lock().unwrap();
                                shared_state.completed = true;
                                if let Some(waker) = shared_state.waker.take() {
                                    waker.wake()
                                }
                                break;
                            },
                        }
                    },
                    Err(err) => println!("recv err: {:?}", err),
                }
            }

            println!("{:?} terminate", tid);
        });

        ConsumerFuture {
            shared_state,
            sender: Some(sender),
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let vec: Vec<u16> = (1..=10).collect();

    let consumer_future_1 = ConsumerFuture::new()
    let consumer_future_2 = ConsumerFuture::new()

    let chans = [
        consumer_future_1.sender,
        consumer_future_2.sender,
    ];

    // let consumer_future = async move || {
    //     join!(consumer_future_1, consumer_future_2)
    // }

    tokio::spawn(consumer_future_1);
    tokio::spawn(consumer_future_2);

    // 主线程负责协调工作线程实现要求的调度策略
    for i in 0..vec.len() {
        thread::sleep(Duration::from_millis(100));
        if let Some(ref chan) = &chans[i % &chan.len()] {
            &chan.send(Some(vec[i]));
        }
    }

    for i in 0..chans.len() {
        if let Some(ref chan) = &chans[i] {
            &chan.send(None);
        }
    }

    // 阻塞, 等待所有 future 结束
    // executor.block_on(consumer_future());
}