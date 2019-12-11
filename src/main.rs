#![allow(unused)]

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    sync::mpsc::{channel, Sender, Receiver},
    task::{Context, Waker, Poll},
    thread,
    time::Duration,
    io,
};

use futures::executor;
use futures::future::join;

pub struct ConsumerFuture {
    shared_state: Arc<Mutex<SharedState>>,
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
    pub fn new(receiver: Receiver<Option<u16>>) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();

        thread::spawn(move || {
            let tid = thread::current().id();
            loop {
                match receiver.recv() {
                    Ok(option) => {
                        match option {
                            Some(elem) => println!("{:?} consume element: {:?}", tid, elem),
                            None => {
                                println!("future <{:?}> state change -> completed", tid);
                                let mut shared_state = thread_shared_state.lock().unwrap();
                                shared_state.completed = true;
                                if let Some(waker) = shared_state.waker.take() {
                                    waker.wake()
                                }
                                break;
                            },
                        }
                    },
                    Err(err) => println!("{:?} recv err: {:?}", tid, err),
                }
            }
        });

        ConsumerFuture {
            shared_state,
        }
    }
}

fn main() {
    let vec: Vec<u16> = (1..=10).collect();

    let (sender_1, receiver_1) = channel();
    let (sender_2, receiver_2) = channel();

    let chans = [
        &sender_1,
        &sender_2,
    ];

    let consumer_future_1 = ConsumerFuture::new(receiver_1);
    let consumer_future_2 = ConsumerFuture::new(receiver_2);

    let waiter = thread::spawn(move || {
        let handler = join(consumer_future_1, consumer_future_2);
        // 阻塞, 等待所有 future 结束
        executor::block_on(handler);

        println!("all future (threads) done");
    });

    // 主线程负责协调工作线程实现要求的调度策略
    for i in 0..vec.len() {
        thread::sleep(Duration::from_millis(100));
        let chan = &chans[i % &chans.len()];
        chan.send(Some(vec[i]));
    }

    for i in 0..chans.len() {
        let chan = &chans[i];
        chan.send(None);
    }

    waiter.join();
}