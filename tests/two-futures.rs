use futures::{executor::block_on, join};
use name_it::name_it;

#[name_it(First)]
async fn first() {
}

#[name_it(Second)]
async fn second() {
}

#[test]
fn test() {
    let first = first();
    let second = second();
    block_on(async { join!(first, second) });
}
