use futures::{pin_mut, FutureExt as _};
use name_it::name_it;

async fn bar() {}

#[name_it(Test)]
async fn has_lifetime(x: &str) -> String {
    bar().await;
    x.to_string()
}

#[test]
fn should_work() {
    let x = String::from("Hi!");
    let y: Test = has_lifetime(x.as_str());
    pin_mut!(y);
    assert_eq!(y.now_or_never().unwrap(), "Hi!");
}
