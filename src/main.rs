// // 创建包含1-10，十个数字的数组，用两个线程依次从数组中获取元素，每100ms 获取一个。这一行为封装成一个 Future， 拿到五个后完成Future。
// // 主线程在两个子线程Future结束后退出。

// #![allow(unused)]

// use std::thread;
// use std::sync::mpsc::channel;
// use std::sync::mpsc::Sender;
// use std::time::Duration;

// fn main() {
//     let vec: Vec<u16> = (1..=10).collect();

//     let mut chans: [Option<Sender<Option<u16>>>; 2] = [None, None];
//     let mut thrs: [Option<thread::JoinHandle<()>>; 2] = [None, None];

//     for i in 0..2 {
//         let (sender, receiver) = channel();
//         chans[i] = Some(sender);

//         // 工作线程负责接收任务
//         let thr = thread::spawn(move || {
//             let tid = thread::current().id();

//             // TODO: use Future
//             loop {
//                 match receiver.recv() {
//                     Ok(option) => {
//                         match option {
//                             Some(elem) => println!("{:?} consume element: {:?}", tid, elem),
//                             None => {
//                                 println!("{:?} consume done", tid);
//                                 break;
//                             },
//                         }
//                     },
//                     Err(err) => println!("recv err: {:?}", err),
//                 }
//             }

//             println!("{:?} terminate", tid);
//         });

//         thrs[i] = Some(thr);
//     }

//     // 主线程负责调度策略
//     for i in 0..vec.len() {
//         thread::sleep(Duration::from_millis(100));
//         if let Some(ref chan) = &chans[i % &chans.len()] {
//             &chan.send(Some(vec[i]));
//         }
//     }

//     // 依次通知结束
//     for i in 0..chans.len() {
//         if let Some(ref chan) = &chans[i] {
//             &chan.send(None);
//         }
//     }

//     // 等待所有工作线程退出
//     for i in 0..thrs.len() {
//         thrs[i].take().unwrap().join();
//     }
// }

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