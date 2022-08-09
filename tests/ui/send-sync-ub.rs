use name_it::name_it;

use futures::prelude::*;
use std::{cell::Cell, rc::Rc, sync::atomic, thread};
type X = Result<&'static [u8], &'static [()]>;

#[name_it(Foo)]
async fn foo(x: Rc<Cell<X>>) -> Rc<Cell<X>> {
    x
}

fn main() {
    let cell = Rc::new(Cell::new(Ok(&[][..])));
    let fut = foo(cell.clone());

    // Foo implements Send, even though it contains an Rc<Cell<...>>
    thread::spawn(|| {
        // get the Rc<Cell<...>> back out of the future
        let cell = fut.now_or_never().unwrap();

        // create a data race
        loop {
            atomic::fence(atomic::Ordering::SeqCst); // make this exploit work with --release, too

            cell.set(Ok(&[][..])); // zero-element `u8` array

            atomic::fence(atomic::Ordering::SeqCst); // make this exploit work with --release, too

            cell.set(Err(&[(); 1000000][..])); // very long `()` array
        }
    });

    // create a data race
    loop {
        atomic::fence(atomic::Ordering::SeqCst); // make this exploit work with --release, too

        // check for `u8` slice with len of the `()` slice (and pointer to either one)
        if let Ok(slice @ [_, ..]) = cell.get() {
            for i in (0..1000000).step_by(10000) {
                dbg!(slice[i]); // create a seg-fault
            }
            return;
        }
    }
}
