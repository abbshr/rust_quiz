extern crate futures;

use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use futures::{Poll, executor};
use futures::task::{Context, Waker};
use core::pin::Pin;
use std::thread;
use std::time::Duration;

struct Consumer<'a> {
    sender: Option<Sender<Option<u16>>>;
    thread_handler: Option<thread::JoinHandle<()>>;
    waker: Option<&'a Waker>;
    finished: bool;
}

impl Future for Consumer<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.finished {
            Poll::Ready(())
        } else {
            match self.waker {
                None => self.waker = Some(cx.waker()),
                _ => (),
            }
            Poll::Pending
        }
    }
}

impl Consumer<'a> {
    fn new() -> Consumer<'a> {
        let consumer = Consumer {
            sender: None;
            thread_handler: None;
            waker: None;
            finished: false;
        };

        let (sender, receiver) = channel();

        consumer.sender = sender;

        let thread_handler = thread::spawn(move || {
            let tid = thread::current().id();

            loop {
                match receiver.recv() {
                    Ok(option) => {
                        match option {
                            Some(elem) => println!("{:?} consume element: {:?}", tid, elem),
                            None => {
                                println!("{:?} consume done", tid);
                                consumer.finished = true;
                                if let Some(waker) = consumer.waker {
                                    waker.wake();
                                }
                                break;
                            },
                        }
                    },
                    Err(err) => {
                        println!("recv err: {:?}", err);
                        consumer.finished = true;
                        if let Some(waker) = consumer.waker {
                            waker.wake();
                        }
                        break;
                    },
                }
            }

            println!("{:?} terminate", tid);
        });

        consumer.thread_handler = thread_handler;
        consumer
    }
}

async waitAll(consumer_1: Consumer<'a>, consumer_2: Consumer<'a>) {
    join!(consumer_1, consumer_2)
}

fn main() {
    let consumer_1 = Consumer::new();
    let consumer_2 = Consumer::new();

    executor.block_on(waitAll(consumer_1, consumer_2));
}