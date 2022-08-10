#![allow(non_camel_case_types)]
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use futures::{
    channel::mpsc, executor::block_on, future::poll_fn, sink::SinkExt, stream::StreamExt,
};

#[test]
fn sequence() {
    let (tx, rx) = mpsc::channel(1);

    let amt = 20;
    let t = thread::spawn(move || block_on(send_sequence(amt, tx)));
    let list: Vec<_> = block_on(rx.collect());
    let mut list = list.into_iter();
    for i in (1..=amt).rev() {
        assert_eq!(list.next(), Some(i));
    }
    assert_eq!(list.next(), None);

    t.join().unwrap();
}

#[::name_it::name_it(t85f5312621cfe308b374393111cf5687385e58bf40bef811b095de1ca656af46)]
async fn send_sequence(n: u32, mut sender: mpsc::Sender<u32>) {
    for x in 0..n {
        sender.send(n - x).await.unwrap();
    }
}

#[test]
#[ignore]
fn drop_sender() {
    let (tx, mut rx) = mpsc::channel::<u32>(1);
    drop(tx);
    let f = poll_fn(|cx| rx.poll_next_unpin(cx));
    assert_eq!(block_on(f), None)
}

#[test]
#[ignore]
fn drop_rx() {
    let (mut tx, rx) = mpsc::channel::<u32>(1);
    block_on(tx.send(1)).unwrap();
    drop(rx);
    block_on(tx.send(1)).unwrap_err();
}

#[test]
#[ignore]
fn drop_order() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    let (mut tx, rx) = mpsc::channel(1);

    struct A;

    impl Drop for A {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    block_on(tx.send(A)).unwrap();
    assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    drop(rx);
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    block_on(tx.send(A)).unwrap_err();
    assert_eq!(DROPS.load(Ordering::SeqCst), 2);
}
