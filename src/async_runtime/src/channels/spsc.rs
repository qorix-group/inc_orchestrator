//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//

use foundation::prelude::*;
use std::{future::Future, sync::Arc, task::Waker};

use crate::{
    futures::{FutureInternalReturn, FutureState},
    scheduler::waker::AtomicWakerStore,
};

///
/// Creates Single Producer Single Consumer channel with [`CHANNEL_SIZE`] capacity
///
pub fn create_channel_default<T: Copy>() -> (Sender<T, CHANNEL_SIZE>, Receiver<T, CHANNEL_SIZE>) {
    let chan = Arc::new(Channel::new());

    (Sender { chan: chan.clone() }, Receiver { chan })
}

///
/// Creates Single Producer Single Consumer channel with `Size` capacity
///
pub fn create_channel<T: Copy, const SIZE: usize>() -> (Sender<T, SIZE>, Receiver<T, SIZE>) {
    let chan = Arc::new(Channel::<T, SIZE>::new());

    (Sender { chan: chan.clone() }, Receiver { chan })
}

pub struct Sender<T: Copy, const SIZE: usize> {
    chan: Arc<Channel<T, SIZE>>,
}

unsafe impl<T: Send + Copy, const SIZE: usize> Send for Sender<T, SIZE> {}

impl<T: Copy, const SIZE: usize> Sender<T, SIZE> {
    ///
    /// Sends value to connected `Receiver`
    ///
    /// # Returns
    ///
    /// * `Ok(())` - when data was send
    /// * `Err(CommonErrors::NoSpaceLeft)` - when no more space available in channel.
    /// * `Err(CommonErrors::GenericError)` - when `Receiver` is disconnected (ie dropped)
    ///
    pub fn send(&self, value: &T) -> Result<(), CommonErrors> {
        self.chan.send(value)
    }
}

impl<T: Copy, const SIZE: usize> Drop for Sender<T, SIZE> {
    fn drop(&mut self) {
        self.chan.sender_dropping();
    }
}

pub struct Receiver<T: Copy, const SIZE: usize> {
    chan: Arc<Channel<T, SIZE>>,
}

impl<T: Copy, const SIZE: usize> Drop for Receiver<T, SIZE> {
    fn drop(&mut self) {
        self.chan.receiver_dropping();
    }
}

unsafe impl<T: Send + Copy, const SIZE: usize> Send for Receiver<T, SIZE> {}

impl<T: Copy, const SIZE: usize> Receiver<T, SIZE> {
    ///
    /// Receive data on a channel
    ///
    ///  # Returns
    ///
    /// * `Some(_)` - when new sample arrived
    /// * `None` - when `Sender` has disconnected and this channel is not usable anymore.
    ///
    pub async fn recv(&mut self) -> Option<T> {
        ReceiverFuture {
            parent: self,
            state: FutureState::default(),
        }
        .await
    }

    fn receive(&self, waker: Waker) -> Result<T, CommonErrors> {
        self.chan.receive(waker)
    }
}

const CHANNEL_SIZE: usize = 8;
const BOTH_IN: u8 = 0;
const SENDER_GONE: u8 = 1;
const RECV_GONE: u8 = 2;

pub(super) struct Channel<T: Copy, const SIZE: usize> {
    queue: spsc::queue::Queue<T, SIZE>,
    waker_store: AtomicWakerStore,
    connected_state: FoundationAtomicU8,
}

impl<T: Copy, const SIZE: usize> Channel<T, SIZE> {
    fn new() -> Self {
        Self {
            queue: spsc::queue::Queue::new(),
            waker_store: AtomicWakerStore::default(),
            connected_state: FoundationAtomicU8::new(BOTH_IN),
        }
    }

    fn sender_dropping(&self) {
        let prev = self.connected_state.swap(SENDER_GONE, std::sync::atomic::Ordering::SeqCst);

        if prev == BOTH_IN {
            // if receiver is still there, notify him
            self.waker_store.take().map_or_else(|| {}, |w| w.wake());
        }
    }

    fn receiver_dropping(&self) {
        self.connected_state.store(RECV_GONE, std::sync::atomic::Ordering::SeqCst);
        let _ = self.waker_store.take();
    }

    ///
    /// Safety: Upper layer needs to assure that there is no other `send` caller at the same time, otherwise this will panic
    ///
    fn send(&self, value: &T) -> Result<(), CommonErrors> {
        // if receiver is gone here,
        if self.connected_state.load(std::sync::atomic::Ordering::Acquire) == RECV_GONE {
            Err(CommonErrors::GenericError)
        } else {
            let res = self.queue.acquire_producer().unwrap().push(value);
            if !res {
                return Err(CommonErrors::NoSpaceLeft);
            }

            // Safety: store makes sure this is not reordered before we push
            if let Some(waker) = self.waker_store.take() {
                waker.wake();
            }

            Ok(())
        }
    }

    ///
    /// Safety: Upper layer needs to assure that there is no other `receive` caller at the same time, otherwise this will panic
    ///
    fn receive(&self, waker: Waker) -> Result<T, CommonErrors> {
        let res = loop {
            let empty = self.queue.is_empty();

            if empty {
                // Register current waker
                let old = self.waker_store.swap(Some(waker.clone()));
                let mut res = Err(CommonErrors::NoData);

                if old.is_some() {
                    // We get woken by "Someone" (because Sender did not took a waker), there still may be an item in queue already but we just return like there is none.
                    // If there was pushed data in between, the user will use current waker to notify us and we will pick work next `poll`
                    break res;
                }

                let state = self.connected_state.load(std::sync::atomic::Ordering::Acquire);

                // There is no old waker, maybe we got notified already and the receiver is drop now
                if SENDER_GONE == state {
                    // yes it's dropped, so we must recheck queue

                    if self.queue.is_empty() {
                        //Sender dropped and no items, we are done
                        res = Err(CommonErrors::AlreadyDone);
                        let _ = self.waker_store.take(); // Clear waker.
                    } else {
                        // Let us process items still
                        continue;
                    }
                }

                break res;
            } else {
                // There is data already, take a piece, clear a waker if it was set and continue as we don't need to register waker really since Future is completed now
                let _ = self.waker_store.take();
                let mut consumer = self.queue.acquire_consumer().unwrap();
                break Ok(consumer.pop().unwrap());
            }
        };

        res
    }
}

struct ReceiverFuture<'a, T: Copy, const SIZE: usize> {
    parent: &'a Receiver<T, SIZE>,
    state: FutureState,
}

impl<T: Copy, const SIZE: usize> Future for ReceiverFuture<'_, T, SIZE> {
    type Output = Option<T>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let res = match self.state {
            FutureState::New | FutureState::Polled => self.parent.receive(cx.waker().clone()).map_or_else(
                |e| {
                    if e == CommonErrors::NoData {
                        FutureInternalReturn::polled()
                    } else {
                        FutureInternalReturn::ready(None)
                    }
                },
                |v| FutureInternalReturn::ready(Some(v)),
            ),
            FutureState::Finished => todo!(),
        };

        self.state.assign_and_propagate(res)
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {

    use super::*;

    use crate::testing::*;
    use testing::prelude::*;

    #[test]
    fn test_channel_ordered_send_receive_works() {
        let (s, mut r) = create_channel_default::<u32>();

        let input = vec![1, 2, 10, 3];
        for e in &input {
            assert!(s.send(e).is_ok());
        }

        let cnt = input.len();
        let mut poller = TestingFuturePoller::new(async move {
            let mut v = vec![];

            for _ in 0..cnt {
                v.push(r.recv().await.unwrap());
            }

            v
        });

        let (waker, _) = get_dummy_task_waker();
        let res = poller.poll_with_waker(&waker);
        assert_poll_ready(res, input);
    }

    #[test]
    fn test_channel_receiver_gone() {
        {
            let (s, _) = create_channel_default::<u32>();

            assert!(s.send(&1).is_err());
        }

        {
            let (s, mut r) = create_channel_default::<u32>();

            assert!(s.send(&1).is_ok());
            {
                let (waker, _) = get_dummy_task_waker();

                let mut poller = TestingFuturePoller::new(async move { r.recv().await.unwrap() });

                let res = poller.poll_with_waker(&waker);
                assert_poll_ready(res, 1);
            }

            // recv was dropped from above scope
            assert!(s.send(&1).is_err());
        }
    }

    #[test]
    fn test_channel_sender_gone() {
        // sender gone but there is data
        {
            let (s, mut r) = create_channel_default::<u32>();

            assert!(s.send(&1).is_ok());
            drop(s);

            {
                let (waker, _) = get_dummy_task_waker();

                let mut poller = TestingFuturePoller::new(async move {
                    let mut v = vec![];
                    for _ in 0..2 {
                        v.push(r.recv().await);
                    }

                    v
                });

                let res = poller.poll_with_waker(&waker);
                assert_poll_ready(res, vec![Some(1), None]); // data before sender is dropped and then None imidietelly
            }
        }
    }

    #[test]
    fn test_channel_receiver_different_wakes() {
        // Data available at first poll
        {
            let (s, mut r) = create_channel_default::<u32>();
            assert!(s.send(&1).is_ok());

            {
                let mut poller = TestingFuturePoller::new(async move {
                    loop {
                        let res = r.recv().await;
                        if res.is_none() {
                            continue;
                        } else {
                            return res.unwrap();
                        }
                    }
                });

                let (waker, tracker) = get_dummy_task_waker();

                {
                    // Poll with data shall get value and not store waker
                    let res = poller.poll_with_waker(&waker);
                    assert_poll_ready(res, 1);

                    drop(waker);
                }

                assert_eq!(1, Arc::strong_count(&tracker));
            }
        }

        // No data at first poll, then data
        {
            let (s, mut r) = create_channel_default::<u32>();

            {
                let mut poller = TestingFuturePoller::new(async move {
                    loop {
                        let res = r.recv().await;
                        if res.is_none() {
                            continue;
                        } else {
                            return res.unwrap();
                        }
                    }
                });

                let (mut waker, tracker) = get_dummy_task_waker();

                {
                    // Poll with data shall not have data and store waker
                    let res = poller.poll_with_waker(&waker);
                    assert!(res.is_pending());
                    drop(waker);
                }

                assert_eq!(2, Arc::strong_count(&tracker));

                // Now set value
                assert!(s.send(&1).is_ok());

                waker = get_waker_from_task(&tracker);

                {
                    // Poll with data shall get value and not store waker
                    let res = poller.poll_with_waker(&waker);
                    assert_poll_ready(res, 1);
                    drop(waker);
                }

                assert_eq!(1, Arc::strong_count(&tracker));
            }
        }
    }

    #[test]
    fn test_channel_send_when_full() {
        let (s, mut r) = create_channel::<u32, 2>();

        // Fill the channel to capacity
        assert!(s.send(&1).is_ok());
        assert!(s.send(&2).is_ok());

        // Attempt to send another value, which should fail
        assert!(s.send(&3).is_err());

        // Consume one value to make space
        let (waker, _) = get_dummy_task_waker();
        let mut poller = TestingFuturePoller::new(async move {
            let ret = r.recv().await.unwrap();
            AlwaysPending {}.await;

            ret
        });
        let _ = poller.poll_with_waker(&waker); // Will return pending

        // Now sending should succeed, as future is keep due to always pending, so 'r' is not dropped
        assert!(s.send(&3).is_ok());
    }
}

#[cfg(test)]
#[cfg(loom)]
mod tests {

    use super::*;
    use crate::testing::*;

    use testing::prelude::*;

    use loom::model::Builder;

    #[test]
    fn test_channel_mt_sender_receiver() {
        let mut builder = Builder::new();
        builder.preemption_bound = Some(3);

        builder.check(|| {
            let (s, mut r) = create_channel_default::<u32>();

            let input = vec![1, 2, 10, 3];

            let handle = loom::thread::spawn(move || {
                let mut poller = TestingFuturePoller::new(async move {
                    let mut v = vec![];

                    loop {
                        let r = r.recv().await;

                        if r.is_none() {
                            break;
                        }
                        v.push(r.unwrap());
                    }

                    v
                });

                let sched = create_mock_scheduler_sync();

                loop {
                    let waker = get_dummy_sync_task_waker(sched.clone());

                    let res = poller.poll_with_waker(&waker);
                    if res.is_ready() {
                        break res;
                    }

                    while !sched.wait_for_wake() {
                        loom::hint::spin_loop();
                    }
                }
            });

            for e in &input {
                assert!(s.send(e).is_ok());
            }

            drop(s);

            let res = handle.join().unwrap();

            assert_poll_ready(res, input);
        });
    }
}
