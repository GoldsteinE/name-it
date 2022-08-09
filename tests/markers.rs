use std::marker::PhantomPinned;

use impls::impls;
use name_it::name_it;
struct NotSend(*const ());
unsafe impl Sync for NotSend {}

struct NotSync(*const ());
unsafe impl Send for NotSync {}

#[name_it(OnlySend)]
async fn only_send(x: NotSync, y: PhantomPinned) -> (NotSync, PhantomPinned) {
    (x, y)
}

#[name_it(OnlySync)]
async fn only_sync(x: NotSend, y: PhantomPinned) -> (NotSend, PhantomPinned) {
    (x, y)
}

#[name_it(Simple)]
async fn simple() {}

#[test]
fn impls_are_correct() {
    assert!(impls!(OnlySend: Send & !Sync & !Unpin));
    assert!(impls!(OnlySync: !Send & Sync & !Unpin));
    // All `async fn` futures are `!Unpin`.
    // This could change in the future (no pun intended):
    // https://github.com/rust-lang/rust/issues/82187
    assert!(impls!(Simple: Send & Sync & !Unpin));
}
