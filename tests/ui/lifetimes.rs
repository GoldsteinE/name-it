use futures::{pin_mut, FutureExt as _};

use name_it::name_it;

async fn foo() {}

#[name_it(Test)]
async fn has_lifetime(x: &str) -> String {
    foo().await;
    x.to_string()
}

fn main() {
    let x = String::from("test");
    let y = has_lifetime(x.as_str());
    pin_mut!(y);
    drop(x);
    y.now_or_never().unwrap();
}
