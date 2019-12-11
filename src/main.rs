#![allow(unused)]

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    sync::mpsc::{channel, Sender, Receiver},
    task::{Context, Waker, Poll},
    thread,
    time::Duration,
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
    pub fn new(atomic_vec: Arc<Mutex<Vec<u16>>>, capacity: usize) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();

        thread::spawn(move || {
            let mut usage = 0;
            let tid = thread::current().id();
            loop {
                thread::sleep(Duration::from_millis(100));
                let mut vec = atomic_vec.lock().unwrap();

                if usage >= capacity / 2  {
                    println!("future <{:?}> state change -> completed", tid);
                    let mut shared_state = thread_shared_state.lock().unwrap();
                    shared_state.completed = true;
                    if let Some(waker) = shared_state.waker.take() {
                        waker.wake()
                    }
                    break;
                }

                let elem = vec.pop();
                usage += 1;
                println!("{:?} consume element: {:?}", tid, elem);
            }
        });

        ConsumerFuture {
            shared_state,
        }
    }
}

fn main() {
    let vec: Vec<u16> = (1..=10).collect();
    let capacity = vec.len();
    let atomic_vec = Arc::new(Mutex::new(vec));

    let consumer_future_1 = ConsumerFuture::new(atomic_vec.clone(), capacity);
    let consumer_future_2 = ConsumerFuture::new(atomic_vec.clone(), capacity);

    let handler = join(consumer_future_1, consumer_future_2);

    executor::block_on(handler);
    println!("all future (threads) done");
}