use futures::{pin_mut, FutureExt as _};

use name_it::name_it;

struct Wrapper<'a>(&'a str);

async fn foo() {}

#[name_it(Test)]
async fn has_lifetime(x: Wrapper<'_>) -> String {
    foo().await;
    x.0.to_string()
}

fn main() {
    let x = String::from("test");
    let y = has_lifetime(Wrapper(x.as_str()));
    pin_mut!(y);
    drop(x);
    y.now_or_never().unwrap();
}
